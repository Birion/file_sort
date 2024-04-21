use std::fs::{copy, create_dir_all, rename};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use glob::glob;
use once_cell::sync::Lazy;
use regex::{Match, Regex};
use serde::Deserialize;

pub use cli::*;
pub use configuration::*;
use parser::*;
use utils::*;

mod parser;
mod cli;
mod configuration;
mod utils;

pub mod prelude {
    pub use crate::get_configuration_file_option;
    pub use crate::perform_processing_based_on_configuration;
}

pub type RulesList = Vec<Rule>;
pub type ArgumentList = Vec<String>;

const WILDCARD: &str = "*";
const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "Ondřej Vágner";
const APPLICATION: &str = "comic_sort";

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "name")]
pub enum TransformativeFunction {
    Last { args: Option<ArgumentList> },
    First { args: Option<ArgumentList> },
}

impl TransformativeFunction {
    pub fn get_dir(&self, root: &Path) -> Result<PathBuf> {
        let path = self.construct_path(root);
        let path_str = path.to_str().unwrap();
        let results: Vec<PathBuf> = glob(path_str)?.map(|x| x.unwrap()).collect();
        if results.is_empty() {
            panic!("Couldn't find any folders fitting the pattern {}", path_str);
        }

        self.get_result_based_on_transformation(results)
    }

    fn construct_path(&self, root: &Path) -> PathBuf {
        let mut path: PathBuf = root.into();
        let args = match self {
            TransformativeFunction::Last { args } => args,
            TransformativeFunction::First { args } => args,
        };
        match args {
            Some(arg) => {
                for x in arg {
                    path.push(x)
                }
            }
            None => path.push(WILDCARD),
        }

        path
    }

    fn get_result_based_on_transformation(&self, results: Vec<PathBuf>) -> Result<PathBuf> {
        match self {
            TransformativeFunction::Last { .. } => Ok(results[results.len() - 1].clone()),
            TransformativeFunction::First { .. } => Ok(results[0].clone()),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Rules {
    SingleRule(RulesList),
    RootRules(Vec<RulesList>),
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Rule {
    pub title: String,
    pub pattern: Option<String>,
    pub patterns: Option<Vec<String>>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_from_array_to_optional_pathbuf")]
    pub directory: Option<PathBuf>,
    pub function: Option<TransformativeFunction>,
    pub processors: Option<ConfigProcessor>,
    #[serde(default)]
    pub root: usize,
    #[serde(default)]
    pub copy: bool,
    #[serde(skip_deserializing)]
    pub old_pattern: String,
    #[serde(skip_deserializing)]
    pub new_pattern: String,
}

impl Rule {
    pub fn make_patterns(&mut self) -> Result<()> {
        if let Some(pattern) = &self.pattern {
            self.old_pattern = clean_pattern(pattern.as_str())?;
            self.new_pattern = extract_pattern(pattern.as_str())?;
        }
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

#[derive(Debug, Clone)]
pub(crate) struct Processor {
    source: PathBuf,
    target: PathBuf,
}

impl Processor {
    fn new(file: &Path) -> Processor {
        Processor {
            source: file.to_path_buf(),
            target: PathBuf::new(),
        }
    }

    fn is_changed(&self) -> Result<bool> {
        let target_filename = self.target_filename()?;
        let source_filename = self.source_filename()?;
        Ok(target_filename != source_filename)
    }

    fn source_filename(&self) -> Result<&str> {
        self.source.file_name()
            .ok_or(anyhow!("No filename found"))
            .and_then(|os_str| os_str.to_str()
                .ok_or(anyhow!("Filename not valid unicode")))
    }

    fn target_filename(&self) -> Result<&str> {
        self.target.file_name()
            .ok_or(anyhow!("No filename found"))
            .and_then(|os_str| os_str.to_str()
                .ok_or(anyhow!("Filename not valid unicode")))
    }

    fn perform_file_action(&self, is_copy_operation: bool) -> Result<()> {
        let is_rename_operation = !is_copy_operation;
        self.perform_file_operation(is_copy_operation, is_rename_operation)
    }

    fn perform_file_operation(&self, is_copy_operation: bool, is_rename_operation: bool) -> Result<()> {
        if is_copy_operation {
            copy(&self.source, &self.target)?;
        }
        if is_rename_operation {
            rename(&self.source, &self.target)?;
        }
        Ok(())
    }

    fn resolve_group_substring(&self, range: Vec<usize>) -> Result<String> {
        let range_start = range[0];
        let range_end = range[0] + range[1];
        Ok(self.source_filename()?[range_start..range_end].to_string())
    }

    fn parse_dir(&self, directory: &Path) -> Result<PathBuf> {
        static GROUP_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r".*<(.*)>.*").unwrap());
        let directory_string = directory.to_str()
            .expect("Failed to convert directory to string");
        if !GROUP_PATTERN.is_match(directory_string) {
            return Ok(directory.to_path_buf());
        }
        let group_match = GROUP_PATTERN.find(directory_string)
            .expect("Failed to find a match").as_str();
        let found_group = GROUP_PATTERN
            .captures(group_match).unwrap().get(1);
        let group_values = self.extract_group_values(found_group);
        let replace_part = self.resolve_group_substring(group_values)?;
        let new_pattern = Regex::new(&format!("<{}>", found_group.unwrap().as_str()))?;
        let dir = new_pattern.replace(directory_string, &replace_part).to_string();
        Ok(PathBuf::from(dir))
    }

    fn extract_group_values(&self, found_group: Option<Match>) -> Vec<usize> {
        let group_values = found_group
            .unwrap()
            .as_str()
            .split(':')
            .map(|res| res.parse().unwrap())
            .collect::<Vec<usize>>();
        group_values
    }

    fn parse_file(&self, pattern: &str) -> Result<String> {
        let source_filename = self.source_filename()?.to_string();
        let r = Regex::new(pattern)?;
        let group = r.captures(&source_filename)
            .expect("No match found").get(0);
        Ok(if let Some(g) = group { g.as_str().to_string() } else { source_filename })
    }

    fn create_and_set_target_directory(&mut self, root: &Path, folder: &Path) -> Result<()> {
        let folder_full_path = full_path(root, folder);
        self.target = self.parse_dir(&folder_full_path).unwrap();

        Ok(create_dir_all(&self.target)?)
    }

    fn make_destination(&self, new_name: &str, root: Option<&Path>, rule: &Rule) -> Result<PathBuf> {
        let mut processed_value: String = self.parse_file(new_name)?;
        let root = match root {
            None => &self.target,
            Some(r) => r,
        };

        if let Some(config_processor) = &rule.processors {
            if config_processor.date_format.is_some() && config_processor.splitter.is_some() {
                process_date(
                    &mut processed_value,
                    config_processor.date_format.as_ref().unwrap(),
                    config_processor.splitter.as_ref().unwrap(),
                    &config_processor.merger,
                )?;
            }

            if let Some(pattern) = &config_processor.pattern {
                process_pattern(&mut processed_value, pattern, &config_processor.replacement)?;
            }
        }

        Ok(root.join(PathBuf::from(processed_value)))
    }
}

