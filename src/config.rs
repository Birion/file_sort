use std::io::Error;
use std::path::PathBuf;

use regex::{Captures, Regex};
use serde::{Deserialize, Deserializer};
use shellexpand::tilde;

use glob::glob;

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
    pub fn get_files(&mut self) -> Result<Vec<PathBuf>, Error> {
        let mut files: Vec<PathBuf> = vec![];
        for ext in ["jpg", "jpeg", "gif", "png"].iter() {
            let y: &PathBuf = &self.download.join("*.".to_owned() + ext);
            for z in glob(y.to_str().unwrap()).unwrap() {
                files.insert(0, z.unwrap());
            }
        }
        Ok(files)
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
    pub processors: Option<Processor>,
    #[serde(skip_deserializing)]
    pub old_pattern: String,
    #[serde(skip_deserializing)]
    pub new_pattern: String,
}

impl Mapping {
    pub fn make_patterns(&mut self) -> Result<(), Error> {
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
pub struct Processor {
    pub splitter: Option<String>,
    #[serde(default = "default_merger")]
    pub merger: Option<String>,
    pub pattern: Option<String>,
    pub format: Option<String>,
    pub replacement: Option<String>,
}

fn default_merger() -> Option<String> {
    Some(String::from("-"))
}

fn from_array<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where D: Deserializer<'de> {
    let p: Vec<String> = Deserialize::deserialize(deserializer)?;

    fn process_path(path: &str) -> String {
        let mut p: String = tilde(path).to_string();
        if p.ends_with(':') {
            p += "\\";
        };
        p
    }

    Ok(p.iter().map(|res| process_path(res.as_str())).collect())
}

fn to_regex<'de, D>(deserializer: D) -> Result<Regex, D::Error>
    where D: Deserializer<'de> {
    let s: String = Deserialize::deserialize(deserializer)?;
    let r: Regex = Regex::new(s.as_str()).unwrap();
    Ok(r)
}
