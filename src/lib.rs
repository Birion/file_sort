use std::fs::{copy, create_dir_all, rename};
use std::path::{Path, PathBuf};

use glob::glob;
use once_cell::sync::Lazy;
use regex::{Match, Regex};
use serde::Deserialize;

pub use cli::*;
pub use configuration::*;
pub use errors::*;
use parser::*;
use utils::*;

mod cli;
mod configuration;
mod errors;
pub mod logging;
mod parser;
mod utils;

pub mod prelude {
    pub use crate::errors::{
        config_parsing_error, directory_not_found_error, file_operation_error, generic_error,
        glob_pattern_error, invalid_filename_error, no_match_error, path_operation_error,
        pattern_extraction_error, pattern_matching_error,
    };
    pub use crate::errors::{Error, Result};
    pub use crate::get_configuration_file_option;
    pub use crate::logging::{format_message, init_default_logger, init_logger, Verbosity};
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
        let path_str = path
            .to_str()
            .ok_or_else(|| invalid_filename_error(path.clone()))?;

        let pattern_results = glob(path_str).map_err(|e| glob_pattern_error(e, path_str))?;

        let results: Vec<PathBuf> = pattern_results
            .map(|res| {
                res.map_err(|e| file_operation_error(e.into_error(), path.clone(), "access"))
            })
            .collect::<std::result::Result<Vec<PathBuf>, Error>>()?;

        if results.is_empty() {
            return Err(directory_not_found_error(path));
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
    /// Process the rule's pattern to extract the old and new patterns
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

    /// Get the source filename as a string
    fn source_filename(&self) -> Result<&str> {
        self.source
            .file_name()
            .ok_or_else(|| path_operation_error(self.source.clone(), "get filename"))
            .and_then(|os_str| {
                os_str
                    .to_str()
                    .ok_or_else(|| invalid_filename_error(self.source.clone()))
            })
    }

    /// Get the target filename as a string
    fn target_filename(&self) -> Result<&str> {
        self.target
            .file_name()
            .ok_or_else(|| path_operation_error(self.target.clone(), "get filename"))
            .and_then(|os_str| {
                os_str
                    .to_str()
                    .ok_or_else(|| invalid_filename_error(self.target.clone()))
            })
    }

    /// Perform the file action (copy or move)
    fn perform_file_action(&self, is_copy_operation: bool) -> Result<()> {
        let is_rename_operation = !is_copy_operation;
        self.perform_file_operation(is_copy_operation, is_rename_operation)
    }

    /// Perform the actual file operation (copy and/or rename)
    fn perform_file_operation(
        &self,
        is_copy_operation: bool,
        is_rename_operation: bool,
    ) -> Result<()> {
        if is_copy_operation {
            copy(&self.source, &self.target)
                .map_err(|e| file_operation_error(e, self.source.clone(), "copy"))?;
        }
        if is_rename_operation {
            rename(&self.source, &self.target)
                .map_err(|e| file_operation_error(e, self.source.clone(), "move"))?;
        }
        Ok(())
    }

    /// Resolve a substring from the source filename based on a range
    fn resolve_group_substring(&self, range: Vec<usize>) -> Result<String> {
        if range.len() < 2 {
            return Err(generic_error(
                "Invalid range format: expected [start, length]",
            ));
        }

        let range_start = range[0];
        let range_end = range[0] + range[1];

        let filename = self.source_filename()?;
        if range_start >= filename.len() || range_end > filename.len() {
            return Err(generic_error(&format!(
                "Range [{}, {}] is out of bounds for filename of length {}",
                range_start,
                range_end,
                filename.len()
            )));
        }

        Ok(filename[range_start..range_end].to_string())
    }

    /// Parse a directory path, resolving any pattern groups
    fn parse_dir(&self, directory: &Path) -> Result<PathBuf> {
        static GROUP_PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r".*<(.*)>.*").expect("Failed to compile regex pattern for GROUP_PATTERN")
        });

        let directory_string = directory
            .to_str()
            .ok_or_else(|| invalid_filename_error(directory.to_path_buf()))?;

        if !GROUP_PATTERN.is_match(directory_string) {
            return Ok(directory.to_path_buf());
        }

        let group_match = GROUP_PATTERN
            .find(directory_string)
            .ok_or_else(|| pattern_extraction_error(directory_string, "Failed to find a match"))?
            .as_str();

        let found_group = GROUP_PATTERN
            .captures(group_match)
            .ok_or_else(|| pattern_extraction_error(group_match, "Failed to capture groups"))?
            .get(1)
            .ok_or_else(|| pattern_extraction_error(group_match, "No capture group found"))?;

        let group_values = self.extract_group_values(Some(found_group));
        let replace_part = self.resolve_group_substring(group_values)?;

        let pattern_str = format!("<{}>", found_group.as_str());
        let new_pattern =
            Regex::new(&pattern_str).map_err(|e| pattern_matching_error(e, &pattern_str))?;

        let dir = new_pattern
            .replace(directory_string, &replace_part)
            .to_string();
        Ok(PathBuf::from(dir))
    }

    /// Extract group values from a regex match, parsing them as usize
    fn extract_group_values(&self, found_group: Option<Match>) -> Vec<usize> {
        match found_group {
            Some(group) => group
                .as_str()
                .split(':')
                .filter_map(|s| s.parse::<usize>().ok())
                .collect(),
            None => Vec::new(),
        }
    }

    /// Parse the source filename using a regex pattern
    fn parse_file(&self, pattern: &str) -> Result<String> {
        let source_filename = self.source_filename()?.to_string();

        let regex = Regex::new(pattern).map_err(|e| pattern_matching_error(e, pattern))?;

        let captures = regex
            .captures(&source_filename)
            .ok_or_else(|| no_match_error(pattern, &source_filename))?;

        let group = captures.get(0);

        Ok(if let Some(g) = group {
            g.as_str().to_string()
        } else {
            source_filename
        })
    }

    /// Create and set the target directory for the file
    fn create_and_set_target_directory(&mut self, root: &Path, folder: &Path) -> Result<()> {
        let folder_full_path = full_path(root, folder);

        self.target = self.parse_dir(&folder_full_path)?;

        create_dir_all(&self.target)
            .map_err(|e| file_operation_error(e, self.target.clone(), "create directory"))?;

        Ok(())
    }

    /// Create a destination path based on the rule and pattern
    fn make_destination(
        &self,
        new_name: &str,
        root: Option<&Path>,
        rule: &Rule,
    ) -> Result<PathBuf> {
        let mut processed_value: String = self.parse_file(new_name)?;
        let root = match root {
            None => &self.target,
            Some(r) => r,
        };

        if let Some(config_processor) = &rule.processors {
            // Process date if both date_format and splitter are provided
            if let (Some(date_format), Some(splitter)) =
                (&config_processor.date_format, &config_processor.splitter)
            {
                process_date(
                    &mut processed_value,
                    date_format,
                    splitter,
                    &config_processor.merger,
                )?;
            }

            // Process pattern if provided
            if let Some(pattern) = &config_processor.pattern {
                process_pattern(&mut processed_value, pattern, &config_processor.replacement)?;
            }
        }

        Ok(root.join(PathBuf::from(processed_value)))
    }
}
