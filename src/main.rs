use std::{fs, io};
use std::fmt::Error;
use std::path::PathBuf;

use json::{JsonValue, parse};
use regex::{Match, Regex};
use shellexpand::tilde;
use chrono::{Utc, TimeZone};
use std::fs::{create_dir_all, rename};

const PATTERN: &str = r".*<(.*)>.*";


struct Parser {
    splitter: Option<String>,
    merger: Option<String>,
    format: Option<String>,
    pattern: Option<Regex>,
    replacement: Option<String>,
}

impl Parser {
    fn new(processors: &JsonValue) -> Parser {
        let pattern: Option<Regex> = {
            if processors.has_key("pattern") {
                Some(
                    Regex::new(
                        processors["pattern"].as_str().or_else(|| Some("")).unwrap()
                    ).unwrap()
                )
            } else { None }
        };

        let replacement: Option<String> = {
            if processors.has_key("replacement") && processors["replacement"].has_key("percent") {
                Some(processors["replacement"]["percent"].to_string())
            } else { None }
        };
        let fmt: Option<String> = {
            if processors.has_key("format") {
                let mut tmp = processors["format"].to_string();
                let y: Regex = Regex::new(r"[{]year[}]").unwrap();
                let m: Regex = Regex::new(r"[{]month[}]").unwrap();
                let d: Regex = Regex::new(r"[{]day[}]").unwrap();
                tmp = y.replace(tmp.as_str(), "%Y").to_string();
                tmp = m.replace(tmp.as_str(), "%m").to_string();
                tmp = d.replace(tmp.as_str(), "%d").to_string();
                Some(tmp)
            } else { None }
        };
        let splitter: Option<String> = {
            if processors.has_key("splitter") {
                Some(processors["splitter"].to_string())
            } else { None }
        };
        let merger: Option<String> = {
            if processors.has_key("merger") {
                Some(processors["merger"].to_string())
            } else { Some("-".to_string()) }
        };

        Parser {
            splitter,
            merger,
            format: fmt,
            pattern,
            replacement,
        }
    }
}


struct Mapping {
    title: String,
    dir: PathBuf,
    original: String,
    new: String,
    parser: Option<Parser>,
}

impl Mapping {
    fn new(config: &JsonValue, root: String) -> Mapping {
        let replacement_pattern: Regex = Regex::new(PATTERN).unwrap();
        let title: String = config["title"].to_string();
        let mut dir: PathBuf = PathBuf::from(root);
        let pattern: &str = config["pattern"].as_str().unwrap();
        let mut original: String = pattern.to_string();
        let mut new: String = pattern.to_string();

        if config["directory"].is_string() {
            dir = dir.join(PathBuf::from(config["directory"].to_string()));
        } else {
            for member in config["directory"].members() {
                dir = dir.join(member.as_str().unwrap());
            }
        }

        if replacement_pattern.is_match(config["pattern"].as_str().unwrap()) {
            let re: Regex = Regex::new(r"[<>]").unwrap();
            original = re.replace_all(pattern, "").parse().unwrap();
            new = replacement_pattern.captures(pattern).unwrap().get(1)
                .map_or(String::new(), |m| m.as_str().to_string());
        }

        let processors: &JsonValue = &config["processors"];
        let parser: Option<Parser> = Some(Parser::new(processors));

        Mapping {
            title,
            dir,
            original,
            new,
            parser,
        }
    }
}


struct FSManager {
    root: PathBuf,
    download: PathBuf,
    files: Vec<PathBuf>,
}

impl FSManager {
    fn new(config: &JsonValue) -> FSManager {
        fn to_vec(config: &JsonValue) -> Vec<String> {
            config
                .members()
                .map(|res| res.as_str().unwrap().to_string())
                .collect()
        }
        let root: PathBuf = FSManager::parse(to_vec(&config["root"]));
        let download: PathBuf = FSManager::parse(to_vec(&config["download"]));
        let files: Vec<PathBuf> = fs::read_dir(&download).unwrap()
            .map(|res| res.map(|e| e.path()).unwrap())
            .collect();
        FSManager {
            root,
            download,
            files,
        }
    }

    fn parse(src: Vec<String>) -> PathBuf {
        fn process_path(path: &str) -> String {
            let mut p: String = tilde(path).to_string();
            if p.ends_with(':') {
                p += "\\";
            };
            p
        }

        let path: PathBuf = src.iter().map(|res| process_path(res)).collect();

        path
    }
}


struct Processor {
    file: PathBuf,
    _target: PathBuf,
}

impl Processor {
    fn new(file: &PathBuf) -> Processor {
        Processor {
            file: file.to_path_buf(),
            _target: PathBuf::new(),
        }
    }

    fn filename(&self) -> &str {
        self.file.file_name().unwrap().to_str().unwrap()
    }

    fn parse_dir(&self, directory: &PathBuf) -> Result<PathBuf, Box<Error>> {
        let replacement_pattern: Regex = Regex::new(PATTERN).unwrap();
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
        let replace_part: &str =
            &String::from(self.file.file_name().unwrap().to_str().unwrap())[start..finish];

        let new_pattern: Regex = Regex::new(format!("<{}>", found_group.unwrap().as_str()).as_str())
            .unwrap();

        let dir: String = new_pattern
            .replace(directory.to_str().unwrap(), replace_part)
            .to_string();

        Ok(PathBuf::from(dir))
    }

    fn parse_file(&self, pattern: &str) -> Result<String, Error> {
        let mut result: String = self.filename().to_string();
        let r: Regex = Regex::new(pattern).unwrap();
        let group: Option<Match> = r.captures(self.filename()).unwrap().get(0);
        if !group.is_none() {
            let group: &str = group.unwrap().as_str();
            result = group.to_string()
        }
        Ok(result)
    }

    fn make_target_dir(&mut self, folder: &PathBuf) -> &mut Processor {
        self._target = self.parse_dir(folder).unwrap();
        let _ = create_dir_all(&self._target);
        self
    }

    fn make_dst(&self, new_name: &String, mapping: &Mapping) -> PathBuf {
        let mut dst: String = self.parse_file(new_name.as_str()).unwrap();

        if mapping.parser.is_some() {
            let p: Option<&Parser> = mapping.parser.as_ref();
            if !p.unwrap().splitter.is_none() {
                let parts: Vec<&str> = dst.split(p.unwrap().splitter.as_ref().unwrap().as_str())
                    .collect();
                let creation_date: String = Utc.timestamp(parts[0].parse().unwrap(), 0)
                    .format(p.unwrap().format.as_ref().unwrap().as_str()).to_string();
                dst = vec![creation_date.as_str(), parts[1]]
                    .join(p.unwrap().merger.as_ref().unwrap().as_str());
            }
            if !p.unwrap().pattern.is_none() {
                let pat = p.unwrap().pattern.as_ref().unwrap();
                dst = pat.replace(dst.as_str(),
                                  p.unwrap().replacement.as_ref().unwrap().as_str())
                    .to_string();
            }
        }

        self._target.join(PathBuf::from(dst))
    }
}

fn process_file(file: &PathBuf, mappings: &Vec<Mapping>, fs: &FSManager) -> Result<(), io::Error> {
    let mut processor: Processor = Processor::new(file);

    for mapping in mappings {
        let map_pattern: Regex = Regex::new(mapping.original.as_str()).unwrap();
        if map_pattern.is_match(&processor.filename()) {
            processor.make_target_dir(&mapping.dir);

            let source: PathBuf = fs.download.join(processor.filename());
            let target: PathBuf = processor.make_dst(&mapping.new, mapping);
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

fn main() {
    let config: JsonValue = parse(
        fs::read_to_string("config.json").unwrap().as_str()
    ).expect("Couldn't read JSON file.");

    let fs: FSManager = FSManager::new(&config);
    let mappings: Vec<Mapping> = config["mappings"].members()
        .map(
            |res|
                Mapping::new(res, fs.root.to_str().unwrap().to_string())
        ).collect();

    for file in &fs.files {
        let _ = process_file(file, &mappings, &fs).unwrap();
    }
}
