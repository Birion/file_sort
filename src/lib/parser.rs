use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Deserializer};
use shellexpand::tilde;

pub fn from_array<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
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

pub fn to_regex<'de, D>(deserializer: D) -> Result<Regex, D::Error>
    where D: Deserializer<'de> {
    let s: String = Deserialize::deserialize(deserializer)?;
    let r: Regex = Regex::new(s.as_str()).unwrap();
    Ok(r)
}


pub fn default_merger() -> Option<String> {
    Some(String::from("-"))
}
