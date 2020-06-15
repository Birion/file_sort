use std::fs::{create_dir_all, rename};
use std::io;
use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use regex::{Match, Regex};

use crate::config::{Config, Mapping};
use crate::config::Processor as ConfigProcessor;

struct Processor {
    file: PathBuf,
    target: PathBuf,
}

impl Processor {
    fn new(file: &PathBuf) -> Processor {
        Processor {
            file: file.to_path_buf(),
            target: PathBuf::new(),
        }
    }

    fn filename(&self) -> &str {
        self.file.file_name().unwrap().to_str().unwrap()
    }

    fn parse_dir(&self, directory: &PathBuf) -> Result<PathBuf, io::Error> {
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

    fn parse_file(&self, pattern: &str) -> Result<String, io::Error> {
        let mut result: String = self.filename().to_string();
        let r: Regex = Regex::new(pattern).unwrap();
        let group: Option<Match> = r.captures(self.filename()).unwrap().get(0);
        if let Some(g) = group {
            result = g.as_str().to_string();
        }
        Ok(result)
    }

    fn make_target_dir(&mut self, root: &PathBuf, folder: &PathBuf) -> &mut Processor {
        let folder = root.join(folder);
        self.target = self.parse_dir(&folder).unwrap();
        let _ = create_dir_all(&self.target);
        self
    }

    fn make_dst(&self, new_name: &str, root: &PathBuf, mapping: &Mapping) -> PathBuf {
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

pub fn process_file(file: &PathBuf, config: &Config) -> Result<(), io::Error> {
    let mut processor: Processor = Processor::new(file);

    for mapping in &config.mappings {
        let map_pattern: Regex = Regex::new(mapping.old_pattern.as_str()).unwrap();
        if map_pattern.is_match(&processor.filename()) {
            processor.make_target_dir(&config.root, &mapping.directory);

            let source: PathBuf = config.download.join(processor.filename());
            let target: PathBuf = processor.make_dst(&mapping.new_pattern, &config.root, &mapping);
            let new_filename: &str = target.file_name().unwrap().to_str().unwrap();
            println!(
                "{file} found! Applying setup for {title}.",
                file = processor.filename(),
                title = mapping.title,
            );
            if new_filename != processor.filename()
            {
                println!("New filename: {}", new_filename)
            }
            println!();

            let _ = rename(source, target);
        }
    }

    Ok(())
}
