extern crate core;

pub mod prelude {
    pub use super::config::read;
    pub use super::structs::{Config, Mapping};
    pub use super::utils::get_matches;
}

mod parser {
    use std::path::PathBuf;

    use serde::{Deserialize, Deserializer};
    use shellexpand::tilde;

    use super::enums::Mappings;
    use super::structs::Mapping;

    fn process_path(path: &str) -> String {
        let mut p: String = tilde(path).to_string();
        if p.ends_with(':') {
            p += "\\";
        };
        p
    }

    pub fn from_array<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        let p: Vec<String> = Deserialize::deserialize(deserializer)?;

        Ok(p.iter().map(|res| process_path(res.as_str())).collect())
    }

    pub fn from_array_opt<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let p: Option<Vec<String>> = Deserialize::deserialize(deserializer)?;

        match p {
            None => Ok(None),
            Some(p) => Ok(Some(
                p.iter().map(|res| process_path(res.as_str())).collect(),
            )),
        }
    }

    pub fn from_arrays<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let p: Vec<Vec<String>> = Deserialize::deserialize(deserializer)?;
        let mut res: Vec<PathBuf> = vec![];

        for x in p {
            res.push(x.iter().map(|x| process_path(x.as_str())).collect())
        }

        Ok(res)
    }

    fn map_patterns_to_mappings(mut mapping: Mapping) -> anyhow::Result<Vec<Mapping>> {
        let mut mappings: Vec<Mapping> = vec![];
        if let Some(ref patterns) = mapping.patterns {
            for pattern in patterns {
                mapping.pattern = Some(pattern.to_string());
                let mut m = mapping.clone();
                m.patterns = None;
                mappings.push(m)
            }
        } else {
            mappings.push(mapping)
        }
        Ok(mappings)
    }

    pub fn parse_mappings<'de, D>(deserializer: D) -> Result<Vec<Mapping>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let p: Mappings = Deserialize::deserialize(deserializer)?;
        let mut res: Vec<Mapping> = vec![];
        match p {
            Mappings::Mapping(m) => {
                for mapping in m {
                    let all_pattern_mappings = map_patterns_to_mappings(mapping).unwrap();
                    res.extend(all_pattern_mappings)
                }
            }
            Mappings::Root(r) => {
                for (idx, root) in r.into_iter().enumerate() {
                    for mut map in root {
                        map.root = idx;
                        let all_pattern_mappings = map_patterns_to_mappings(map).unwrap();
                        res.extend(all_pattern_mappings)
                    }
                }
            }
        }
        res.dedup();
        Ok(res)
    }

    pub fn default_merger() -> Option<String> {
        Some(String::from("-"))
    }
}

mod enums {
    use std::path::{Path, PathBuf};

    use anyhow::Result;
    use glob::glob;
    use serde::Deserialize;

    use crate::structs::Mapping;

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    #[serde(rename_all = "lowercase")]
    #[serde(tag = "name")]
    pub enum Function {
        Last { args: Option<Vec<String>> },
        First { args: Option<Vec<String>> },
    }

    impl Function {
        pub fn get_dir(&self, root: &Path) -> Result<PathBuf> {
            let mut path: PathBuf = root.into();
            let args = match self {
                Function::Last { args } => args,
                Function::First { args } => args,
            };
            match args {
                Some(arg) => {
                    for x in arg {
                        path.push(x)
                    }
                }
                None => path.push("*"),
            }
            let p = path.to_str().unwrap();

            let results: Vec<PathBuf> = glob(p)?.map(|x| x.unwrap()).collect();

            if results.is_empty() {
                panic!("Couldn't find any folders fitting the pattern {p}")
            }

            match self {
                Function::Last { .. } => Ok(results[results.len() - 1].clone()),
                Function::First { .. } => Ok(results[0].clone()),
            }
        }
    }

    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    pub enum Mappings {
        Mapping(Vec<Mapping>),
        Root(Vec<Vec<Mapping>>),
    }
}

mod structs {
    use std::fs;
    use std::fs::{create_dir_all, rename};
    use std::path::{Path, PathBuf};

    use anyhow::Result;
    use chrono::{TimeZone, Utc};
    use colored::Colorize;
    use glob::glob;
    use regex::{Captures, Match, Regex};
    use serde::Deserialize;
    use serde_yaml::from_str;

    use super::enums::*;
    use super::parser::*;

    #[derive(Deserialize, Debug)]
    pub struct Config {
        #[serde(deserialize_with = "from_arrays")]
        pub root: Vec<PathBuf>,
        #[serde(deserialize_with = "from_array")]
        pub download: PathBuf,
        #[serde(deserialize_with = "parse_mappings")]
        pub mappings: Vec<Mapping>,
        #[serde(skip_deserializing)]
        pub files: Vec<PathBuf>,
    }

    impl Config {
        pub fn get_files(&mut self) -> Result<()> {
            let y: &PathBuf = &self.download.join("*");
            for z in glob(y.to_str().unwrap())? {
                self.files.insert(0, z?);
            }
            Ok(())
        }

        pub fn load(file: PathBuf) -> Result<Config> {
            let config: Config = from_str(fs::read_to_string(file.to_str().unwrap())?.as_str())
                .expect("Couldn't read YAML file");
            Ok(config)
        }

        pub fn process(&self, file: &Path, dry: bool) -> Result<()> {
            let mut processor: Processor = Processor::new(file);

            for mapping in &self.mappings {
                let root = &self.root[mapping.root];
                let map_pattern: Regex = Regex::new(mapping.old_pattern.as_str())?;
                if map_pattern.is_match(processor.filename()) {
                    let dir = match mapping.directory.clone() {
                        None => PathBuf::from(&mapping.title),
                        Some(d) => d,
                    };
                    processor.make_target_dir(root, &dir);

                    let source: &Path = &self.download.join(processor.filename());

                    let target = match &mapping.function {
                        None => processor.make_dst(&mapping.new_pattern, None, mapping)?,
                        Some(func) => match func {
                            &_ => {
                                let temp_root =
                                    processor.make_dst(&mapping.new_pattern, None, mapping)?;
                                let r = func.get_dir(temp_root.parent().unwrap()).unwrap();
                                processor.make_dst(&mapping.new_pattern, Some(&r), mapping)?
                            }
                        },
                    };

                    let new_filename: &str = target.file_name().unwrap().to_str().unwrap();
                    println!(
                        "{file} found! Applying setup for {title}.",
                        file = processor.filename().bold(),
                        title = mapping.title.bold().blue(),
                    );
                    if new_filename != processor.filename() {
                        println!("New filename: {}", new_filename.bold().red())
                    }
                    println!();

                    if !dry {
                        let _ = rename(source, target);
                    }
                }
            }

            Ok(())
        }
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct Mapping {
        pub title: String,
        // #[serde(deserialize_with = "to_regex")]
        pub pattern: Option<String>,
        pub patterns: Option<Vec<String>>,
        #[serde(default)]
        #[serde(deserialize_with = "from_array_opt")]
        pub directory: Option<PathBuf>,
        pub function: Option<Function>,
        pub processors: Option<ConfigProcessor>,
        #[serde(default)]
        pub root: usize,
        #[serde(skip_deserializing)]
        pub old_pattern: String,
        #[serde(skip_deserializing)]
        pub new_pattern: String,
    }

    impl Mapping {
        pub fn make_patterns(&mut self) -> Result<()> {
            if let Some(pattern) = &self.pattern {
                self.old_pattern = {
                    let x: Regex = Regex::new(r"[<>]")?;
                    x.replace_all(pattern.as_str(), "").to_string()
                };
                self.new_pattern = {
                    let replacement: Regex = Regex::new(r".*<(.*)>.*")?;
                    let x: Option<Captures> = replacement.captures(pattern.as_str());
                    match x {
                        Some(x) => x.get(1).unwrap().as_str().to_string(),
                        None => pattern.to_string(),
                    }
                }
            };
            Ok(())
        }
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct ConfigProcessor {
        pub splitter: Option<String>,
        #[serde(default = "default_merger")]
        pub merger: Option<String>,
        pub pattern: Option<String>,
        pub date_format: Option<String>,
        pub replacement: Option<String>,
    }

    #[derive(Debug)]
    pub(crate) struct Processor {
        file: PathBuf,
        target: PathBuf,
    }

    impl Processor {
        fn new(file: &Path) -> Processor {
            Processor {
                file: file.to_path_buf(),
                target: PathBuf::new(),
            }
        }

        fn filename(&self) -> &str {
            self.file.file_name().unwrap().to_str().unwrap()
        }

        fn parse_dir(&self, directory: &Path) -> Result<PathBuf> {
            let replacement_pattern: Regex = Regex::new(r".*<(.*)>.*")?;
            let dir: &str = directory.to_str().unwrap();
            if !replacement_pattern.is_match(dir) {
                return Ok(directory.to_path_buf());
            }
            let replacement: Match = replacement_pattern.find(dir).unwrap();
            let found_group: Option<Match> = replacement_pattern
                .captures(replacement.as_str())
                .unwrap()
                .get(1);
            let rg: Vec<usize> = found_group
                .map(|res| res.as_str())
                .unwrap()
                .split(':')
                .map(|res| res.parse().unwrap())
                .collect();

            let start = rg[0];
            let finish = rg[0] + rg[1];
            let replacer = String::from(self.filename());
            let replace_part = replacer[start..finish].to_string();

            let new_pattern: Regex =
                Regex::new(format!("<{}>", found_group.unwrap().as_str()).as_str())?;

            let dir: String = new_pattern
                .replace(directory.to_str().unwrap(), replace_part)
                .to_string();

            Ok(PathBuf::from(dir))
        }

        fn parse_file(&self, pattern: &str) -> Result<String> {
            let mut result: String = self.filename().to_string();
            let r: Regex = Regex::new(pattern)?;
            let group: Option<Match> = r.captures(self.filename()).unwrap().get(0);
            if let Some(g) = group {
                result = g.as_str().to_string();
            }
            Ok(result)
        }

        fn make_target_dir(&mut self, root: &Path, folder: &PathBuf) {
            let folder = root.join(folder);
            self.target = self.parse_dir(&folder).unwrap();
            let _ = create_dir_all(&self.target);
        }

        fn make_dst(
            &self,
            new_name: &str,
            root: Option<&Path>,
            mapping: &Mapping,
        ) -> Result<PathBuf> {
            let mut dst: String = self.parse_file(new_name)?;
            let root = match root {
                None => &self.target,
                Some(r) => r,
            };

            if mapping.processors.is_some() {
                if let Some(p) = &mapping.processors {
                    if let Some(fmt) = &p.date_format {
                        if let Some(splitter) = &p.splitter {
                            let parts: Vec<&str> = if splitter.contains('%') {
                                let mut dt = Utc::now().date_naive();
                                let mut _fmt = dt.format(splitter).to_string();
                                while !dst.contains(&_fmt) {
                                    dt = dt.pred_opt().unwrap();
                                    _fmt = dt.format(splitter).to_string();
                                }
                                dst.split(&_fmt).collect()
                            } else {
                                dst.split(splitter).collect()
                            };
                            let creation_date: String = Utc
                                .timestamp_opt(parts[0].parse()?, 0)
                                .unwrap()
                                .format(fmt)
                                .to_string();
                            dst = vec![creation_date.as_str(), parts[1]]
                                .join(p.merger.as_ref().unwrap().as_str());
                        }
                    }
                    if let Some(pattern) = &p.pattern {
                        let pattern = Regex::new(pattern)?;
                        if let Some(replacement) = &p.replacement {
                            dst = pattern.replace(dst.as_str(), replacement).to_string();
                        }
                    }
                }
            }

            Ok(root.join(PathBuf::from(dst)))
        }
    }
}

mod config {
    use std::fs;
    use std::path::PathBuf;

    use anyhow::Result;
    use directories::ProjectDirs;

    pub fn read(config: PathBuf) -> Result<PathBuf> {
        if !&config.exists() {
            let folder = ProjectDirs::from("com", "Ondřej Vágner", "comic_sort").unwrap();
            if !folder.config_dir().exists() {
                fs::create_dir_all(folder.config_dir())?;
            }
            Ok(folder.config_dir().join(&config))
        } else {
            Ok(config)
        }
    }
}

mod utils {
    use anyhow::Result;
    use clap::{
        command, crate_authors, crate_description, crate_name, crate_version, Arg, ArgMatches,
    };

    pub fn get_matches() -> Result<ArgMatches> {
        let matches: ArgMatches = command!()
            .author(crate_authors!())
            .about(crate_description!())
            .name(crate_name!())
            .version(crate_version!())
            .arg(
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .help("Read from a specific config file")
                    .default_value("config.yaml"),
            )
            .arg(
                Arg::new("dry")
                    .short('n')
                    .help("Run without moving any files")
                    .num_args(0),
            )
            .get_matches();
        Ok(matches)
    }
}
