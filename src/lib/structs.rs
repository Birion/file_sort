use std::error::Error;
use std::fs;
use std::fs::create_dir_all;
use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use glob::glob;
use regex::{Captures, Match, Regex};
use serde::Deserialize;
use serde_yaml::from_str;

use super::parser::*;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(deserialize_with = "from_array")]
    pub root: PathBuf,
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
}

#[derive(Deserialize, Debug)]
pub struct Mapping {
    pub title: String,
    #[serde(deserialize_with = "to_regex")]
    pub pattern: Regex,
    #[serde(deserialize_with = "from_array")]
    pub directory: PathBuf,
    pub function: Option<String>,
    pub processors: Option<ConfigProcessor>,
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

pub(crate) struct Processor {
    file: PathBuf,
    target: PathBuf,
}

impl Processor {
    pub(crate) fn new(file: &PathBuf) -> Processor {
        Processor {
            file: file.to_path_buf(),
            target: PathBuf::new(),
        }
    }

    pub(crate) fn filename(&self) -> &str {
        self.file.file_name().unwrap().to_str().unwrap()
    }

    fn parse_dir(&self, directory: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
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
            .map(|res| res.parse::<usize>().unwrap()).collect();

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

    pub(crate) fn make_target_dir(&mut self, root: &PathBuf, folder: &PathBuf) -> &mut Processor {
        let folder = root.join(folder);
        self.target = self.parse_dir(&folder).unwrap();
        let _ = create_dir_all(&self.target);
        self
    }

    pub(crate) fn make_dst(&self, new_name: &str, root: &PathBuf, mapping: &Mapping) -> PathBuf {
        let mut dst: String = self.parse_file(new_name).unwrap();
        let root = root.join(&self.target);

        if mapping.processors.is_some() {
            let p: Option<&ConfigProcessor> = mapping.processors.as_ref();
            if p.unwrap().splitter.is_some() {
                let parts: Vec<&str> = dst.split(p.unwrap().splitter.as_ref().unwrap().as_str())
                    .collect();
                let creation_date: String = Utc.timestamp(parts[0].parse().unwrap(), 0)
                    .format(p.unwrap().format.as_ref().unwrap().as_str()).to_string();
                dst = vec![creation_date.as_str(), parts[1]]
                    .join(p.unwrap().merger.as_ref().unwrap().as_str());
            }
            if p.unwrap().pattern.is_some() {
                let pat = p.unwrap().pattern.as_ref().unwrap();
                dst = pat.replace(dst.as_str(),
                                  p.unwrap().replacement.as_ref().unwrap().as_str());
            }
        }

        root.join(PathBuf::from(dst))
    }
}