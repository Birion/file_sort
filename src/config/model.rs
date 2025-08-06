//! Configuration data structures
//!
//! This module contains the data structures for configuration.

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::discovery::ContentCondition;
use crate::path_gen::FolderFunction;
use crate::utils::{clean_pattern, extract_pattern};

/// Configuration for the file sorting application
///
/// Contains the root directories, download directory, rules, and files to process.
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// Root directories where files will be moved or copied to
    #[serde(deserialize_with = "deserialize_from_arrays_to_pathbuf_vec")]
    pub root: Vec<PathBuf>,
    /// Directory to scan for files to process
    #[serde(deserialize_with = "deserialize_from_array_to_pathbuf")]
    pub download: PathBuf,
    /// Rules for sorting files
    #[serde(deserialize_with = "parse_rules")]
    pub rules: RulesList,
    /// Files found in the download directory
    #[serde(skip_deserializing)]
    pub files: Vec<PathBuf>,
    /// Path to parent configuration file for inheritance
    #[serde(default)]
    pub parent: Option<String>,
}

impl Config {
    /// Validates the configuration
    ///
    /// This method performs comprehensive validation of the configuration:
    /// - Checks that required fields are present
    /// - Validates that paths exist and are accessible (if check_paths is true)
    /// - Ensures rules are properly formatted
    ///
    /// # Arguments
    /// * `check_paths` - Whether to check if paths exist and are accessible
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error with a helpful message
    ///
    /// # Errors
    /// Returns an error with a detailed message if validation fails
    pub fn validate(&self, check_paths: bool) -> Result<()> {
        // Validate root directories
        if self.root.is_empty() {
            return Err(anyhow!(
                "No root directories specified in configuration. At least one root directory is required."
            ));
        }

        if check_paths {
            for (index, path) in self.root.iter().enumerate() {
                if !path.exists() {
                    return Err(anyhow!(
                        "Root directory {} at index {} does not exist: {}",
                        path.display(),
                        index,
                        "Please check the path and ensure it exists."
                    ));
                }

                if !path.is_dir() {
                    return Err(anyhow!(
                        "Root path {} at index {} is not a directory: {}",
                        path.display(),
                        index,
                        "Please specify a valid directory path."
                    ));
                }
            }

            // Validate download directory
            if !self.download.exists() {
                return Err(anyhow!(
                    "Download directory does not exist: {}\n{}",
                    self.download.display(),
                    "Please check the path and ensure it exists."
                ));
            }

            if !self.download.is_dir() {
                return Err(anyhow!(
                    "Download path is not a directory: {}\n{}",
                    self.download.display(),
                    "Please specify a valid directory path."
                ));
            }
        }

        // Validate rules
        if self.rules.is_empty() {
            return Err(anyhow!(
                "No rules specified in configuration. At least one rule is required."
            ));
        }

        // Validate each rule
        for (index, rule) in self.rules.iter().enumerate() {
            // Check rule title
            if rule.title.trim().is_empty() {
                return Err(anyhow!(
                    "Rule at index {} has an empty title. Each rule must have a title.",
                    index
                ));
            }

            // Check that either pattern or patterns is specified
            if rule.pattern.is_none() && rule.patterns.is_none() {
                return Err(anyhow!(
                    "Rule '{}' has no pattern or patterns specified. Each rule must have at least one pattern.",
                    rule.title
                ));
            }

            // Check that the root index is valid
            if rule.root >= self.root.len() {
                return Err(anyhow!(
                    "Rule '{}' references root index {} which is out of bounds (max index: {}).",
                    rule.title,
                    rule.root,
                    self.root.len() - 1
                ));
            }

            // If a directory is specified and we're checking paths, check that it's valid
            if check_paths && let Some(dir) = &rule.directory {
                let full_path = self.root[rule.root].join(dir);
                if !full_path.exists() && rule.function.is_none() {
                    // Only warn if no transformative function is specified
                    // as the function might create the directory
                    log::debug!(
                        "Warning: Directory for rule '{}' does not exist: {}",
                        rule.title,
                        full_path.display()
                    );
                }
            }
        }

        Ok(())
    }
}

/// Represents a file format conversion configuration
///
/// Specifies the source and target formats for file conversion
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct FormatConversion {
    /// Source file format (e.g., "jpg", "png", "utf-8")
    #[serde(alias = "from")]
    pub source_format: String,
    /// Target file format (e.g., "png", "webp", "utf-16")
    #[serde(alias = "to")]
    pub target_format: String,
    /// Optional resize dimensions for image conversion (width, height)
    pub resize: Option<(u32, u32)>,
}

/// Configuration for processing file paths
///
/// Defines how file paths should be processed, including date formatting,
/// pattern replacement, and file format conversion.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ConfigProcessor {
    /// String to split the filename with
    pub splitter: Option<String>,
    /// String to merge parts of the filename with
    #[serde(default = "default_merger")]
    pub merger: Option<String>,
    /// Regex pattern to match in the filename
    pub pattern: Option<String>,
    /// Format string for date processing
    pub date_format: Option<String>,
    /// Replacement string for pattern matching
    pub replacement: Option<String>,
    /// File format conversion configuration
    pub format_conversion: Option<FormatConversion>,
}

/// Default merger function for ConfigProcessor
fn default_merger() -> Option<String> {
    Some(" ".to_string())
}

/// Default value for match_all_conditions in Rule
fn default_match_all() -> bool {
    true
}

/// A list of rules for file sorting.
///
/// This type is used throughout the application to represent collections of sorting rules.
pub type RulesList = Vec<Rule>;

/// Represents different types of rule configurations
///
/// This enum allows for both single rule lists and multiple rule lists
/// organised by root directories.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Rules {
    /// A single list of rules
    SingleRule(RulesList),
    /// Multiple lists of rules organised by root directories
    RootRules(Vec<RulesList>),
}

/// Represents a rule for file sorting
///
/// A rule defines how files should be matched and where they should be moved or copied.
/// Rules can match files based on filename patterns or content-based conditions.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Rule {
    /// The title of the rule
    pub title: String,
    /// The pattern to match files against
    pub pattern: Option<String>,
    /// Multiple patterns to match files against
    pub patterns: Option<Vec<String>>,
    /// Content-based conditions to match files against
    pub content_conditions: Option<Vec<ContentCondition>>,
    /// Whether all content conditions must match (AND logic) or any can match (OR logic)
    #[serde(default = "default_match_all")]
    pub match_all_conditions: bool,
    /// The directory to move or copy files to
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_from_array_to_optional_pathbuf")]
    pub directory: Option<PathBuf>,
    /// Optional transformative function to apply to the directory
    pub function: Option<FolderFunction>,
    /// Optional processors to apply to the file path
    pub processors: Option<ConfigProcessor>,
    /// The index of the root directory to use
    #[serde(default)]
    pub root: usize,
    /// Whether to copy the file instead of moving it
    #[serde(default)]
    pub copy: bool,
    /// The processed pattern without angle brackets
    #[serde(skip_deserializing, skip_serializing)]
    pub old_pattern: String,
    /// The extracted pattern from between angle brackets
    #[serde(skip_deserializing, skip_serializing)]
    pub new_pattern: String,
}

impl Rule {
    /// Processes the rule's pattern to extract the old and new patterns
    ///
    /// This method extracts patterns from the rule's pattern string. It sets:
    /// - `old_pattern`: The pattern with angle brackets removed
    /// - `new_pattern`: The content between angle brackets
    ///
    /// # Returns
    /// * `Result<()>` - Ok if pattern processing succeeds, or an error
    ///
    /// # Errors
    /// * Returns an error if pattern cleaning or extraction fails
    pub fn make_patterns(&mut self) -> Result<()> {
        if let Some(pattern) = &self.pattern {
            self.old_pattern = clean_pattern(pattern.as_str())?;
            self.new_pattern = extract_pattern(pattern.as_str())?;
        }
        Ok(())
    }
}

// Note: The following deserialization functions are referenced in the struct definitions
// but are defined in the loader.rs file.
use super::loader::{
    deserialize_from_array_to_optional_pathbuf, deserialize_from_array_to_pathbuf,
    deserialize_from_arrays_to_pathbuf_vec, parse_rules,
};
