pub mod prelude {
    pub use super::config::read;
    pub use super::structs::{Config, Mapping};
    pub use super::utils::get_matches;
}

mod parser {
    use std::path::PathBuf;

    use regex::Regex;
    use serde::{Deserialize, Deserializer};
    use shellexpand::tilde;

    fn process_path(path: &str) -> String {
        let mut p: String = tilde(path).to_string();
        if p.ends_with(':') {
            p += "\\";
        };
        p
    }

    pub fn from_array<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
        where D: Deserializer<'de> {
        let p: Vec<String> = Deserialize::deserialize(deserializer)?;

        Ok(p.iter().map(|res| process_path(res.as_str())).collect())
    }

    pub fn from_array_opt<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
        where D: Deserializer<'de> {
        let p: Option<Vec<String>> = Deserialize::deserialize(deserializer)?;

        match p {
            None => { Ok(None) }
            Some(p) => {
                Ok(Some(p.iter().map(|res| process_path(res.as_str())).collect()))
            }
        }
    }

    pub fn to_regex<'de, D>(deserializer: D) -> Result<Regex, D::Error>
        where D: Deserializer<'de> {
        let s: String = Deserialize::deserialize(deserializer)?;
        let r: Regex = Regex::new(s.as_str()).unwrap();
        Ok(r)
    }

    pub fn from_arrays<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
        where D: Deserializer<'de> {
        let p: Vec<Vec<String>> = Deserialize::deserialize(deserializer)?;
        let mut res: Vec<PathBuf> = vec![];

        for x in p {
            res.push(x.iter().map(|x| process_path(x.as_str())).collect())
        }

        Ok(res)
    }

    pub fn default_merger() -> Option<String> {
        Some(String::from("-"))
    }
}

mod structs {
    use std::error::Error;
    use std::fs;
    use std::fs::{create_dir_all, rename};
    use std::path::{Path, PathBuf};

    use chrono::{TimeZone, Utc};
    use colored::Colorize;
    use glob::glob;
    use regex::{Captures, Match, Regex};
    use serde::Deserialize;
    use serde_yaml::from_str;

    use super::parser::*;

    #[derive(Deserialize, Debug)]
    pub struct Config {
        #[serde(deserialize_with = "from_arrays")]
        pub root: Vec<PathBuf>,
        #[serde(deserialize_with = "from_array")]
        pub download: PathBuf,
        pub mappings: Vec<Mapping>,
        #[serde(skip_deserializing)]
        pub files: Vec<PathBuf>,
    }

    impl Config {
        pub fn get_files(&mut self) -> Result<(), Box<dyn Error>> {
            for ext in ["jpg", "jpeg", "gif", "png"].iter() {
                let y: &PathBuf = &self.download.join("*.".to_owned() + ext);
                for z in glob(y.to_str().unwrap()).unwrap() {
                    self.files.insert(0, z.unwrap());
                }
            }
            Ok(())
        }

        pub fn load(file: PathBuf) -> Result<Config, Box<dyn Error>> {
            let config: Config = from_str(
                fs::read_to_string(file.to_str().unwrap())?.as_str())
                .expect("Couldn't read YAML file");
            Ok(config)
        }

        pub fn process(&self, file: &Path) -> Result<(), Box<dyn Error>> {
            let mut processor: Processor = Processor::new(file);

            for mapping in &self.mappings {
                let root = &self.root[mapping.root];
                let map_pattern: Regex = Regex::new(mapping.old_pattern.as_str())?;
                if map_pattern.is_match(processor.filename()) {
                    let dir = match mapping.directory.clone() {
                        None => { PathBuf::from(&mapping.title) }
                        Some(d) => { d }
                    };
                    processor.make_target_dir(root, &dir);

                    let source: &Path = &self.download.join(processor.filename());

                    let target = match &mapping.function {
                        None => {
                            processor.make_dst(&mapping.new_pattern, root, mapping)?
                        }
                        Some(func) => match func.as_str() {
                            &_ => {
                                processor.make_dst(&mapping.new_pattern, root, mapping)?
                            }
                        }
                    };

                    let new_filename: &str = target.file_name().unwrap().to_str().unwrap();
                    println!(
                        "{file} found! Applying setup for {title}.",
                        file = processor.filename().bold(),
                        title = mapping.title.bold().blue(),
                    );
                    if new_filename != processor.filename()
                    {
                        println!("New filename: {}", new_filename.bold().red())
                    }
                    println!();

                    let _ = rename(source, target);
                }
            }

            Ok(())
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Mapping {
        pub title: String,
        #[serde(deserialize_with = "to_regex")]
        pub pattern: Regex,
        #[serde(default)]
        #[serde(deserialize_with = "from_array_opt")]
        pub directory: Option<PathBuf>,
        pub function: Option<String>,
        pub processors: Option<ConfigProcessor>,
        #[serde(default)]
        pub root: usize,
        #[serde(skip_deserializing)]
        pub old_pattern: String,
        #[serde(skip_deserializing)]
        pub new_pattern: String,
    }

    impl Mapping {
        pub fn make_patterns(&mut self) -> Result<(), Box<dyn Error>> {
            self.old_pattern = {
                let x: Regex = Regex::new(r"[<>]").unwrap();
                x.replace_all(self.pattern.as_str(), "").to_string()
            };
            self.new_pattern = {
                let replacement: Regex = Regex::new(r".*<(.*)>.*").unwrap();
                let x: Option<Captures> = replacement.captures(self.pattern.as_str());
                match x {
                    Some(x) => x.get(1).unwrap().as_str().to_string(),
                    None => self.pattern.to_string()
                }
            };
            Ok(())
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct ConfigProcessor {
        pub splitter: Option<String>,
        #[serde(default = "default_merger")]
        pub merger: Option<String>,
        pub pattern: Option<String>,
        pub format: Option<String>,
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

        fn parse_dir(&self, directory: &Path) -> Result<PathBuf, Box<dyn Error>> {
            let replacement_pattern: Regex = Regex::new(r".*<(.*)>.*").unwrap();
            let dir: &str = directory.to_str().unwrap();
            if !replacement_pattern.is_match(dir) {
                return Ok(directory.to_path_buf());
            }
            let replacement: Match = replacement_pattern.find(dir).unwrap();
            let found_group: Option<Match> = replacement_pattern
                .captures(replacement.as_str()).unwrap().get(1);
            let rg: Vec<usize> = found_group
                .map(|res| res.as_str()).unwrap().split(':')
                .map(|res| res.parse().unwrap()).collect();

            let start: usize = rg[0];
            let finish: usize = rg[0] + rg[1];
            let replacer: String = String::from(self.filename());
            let replace_part: &str = replacer[start..finish].as_ref();

            let new_pattern: Regex = Regex::new(
                format!("<{}>", found_group.unwrap().as_str()).as_str()
            )
                .unwrap();

            let dir: String = new_pattern
                .replace(directory.to_str().unwrap(), replace_part)
                .to_string();

            Ok(PathBuf::from(dir))
        }

        fn parse_file(&self, pattern: &str) -> Result<String, Box<dyn Error>> {
            let mut result: String = self.filename().to_string();
            let r: Regex = Regex::new(pattern).unwrap();
            let group: Option<Match> = r.captures(self.filename()).unwrap().get(0);
            if let Some(g) = group {
                result = g.as_str().to_string();
            }
            Ok(result)
        }

        fn make_target_dir(&mut self, root: &Path, folder: &PathBuf) -> &mut Processor {
            let folder = root.join(folder);
            self.target = self.parse_dir(&folder).unwrap();
            let _ = create_dir_all(&self.target);
            self
        }

        fn make_dst(&self, new_name: &str, root: &Path, mapping: &Mapping) -> Result<PathBuf, Box<dyn Error>> {
            let mut dst: String = self.parse_file(new_name)?;
            let root = root.join(&self.target);

            if mapping.processors.is_some() {
                let processor = &mapping.processors;
                if let Some(p) = processor {
                    let fmt = p.format.as_ref().unwrap();
                    if let Some(splitter) = &p.splitter {
                        let parts: Vec<&str> =
                            if splitter.contains('%') {
                                let mut dt = Utc::today();
                                let mut _fmt = dt.format(splitter).to_string();
                                while !dst.contains(&_fmt) {
                                    dt = dt.pred();
                                    _fmt = dt.format(splitter).to_string();
                                }
                                dst.split(&_fmt).collect()
                            } else {
                                dst.split(splitter).collect()
                            };
                        let creation_date: String = Utc.timestamp(parts[0].parse().unwrap(), 0)
                            .format(fmt).to_string();
                        dst = vec![creation_date.as_str(), parts[1]]
                            .join(p.merger.as_ref().unwrap().as_str());
                    }
                    if let Some(pattern) = &p.pattern {
                        let replacement = p.replacement.as_ref().unwrap();
                        dst = pattern.replace(dst.as_str(), replacement);
                    }
                }
            }

            Ok(root.join(PathBuf::from(dst)))
        }
    }
}

mod config {
    use std::error::Error;
    use std::fs;
    use std::path::PathBuf;

    use directories::ProjectDirs;

    pub fn read(config: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
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
    use std::error::Error;

    use clap::{app_from_crate, Arg, ArgMatches, crate_authors, crate_description, crate_name, crate_version};

    pub fn get_matches() -> Result<ArgMatches<'static>, Box<dyn Error>> {
        let matches: ArgMatches = app_from_crate!()
            .arg(Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Read from a specific config file")
                .default_value("config.yaml")
            )
            .get_matches();
        Ok(matches)
    }
}