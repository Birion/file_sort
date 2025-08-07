//! Configuration loading functionality
//!
//! This module contains functions for loading and validating configuration.

use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use log::{debug, error, info};
use serde::Deserialize;
use serde_yaml::from_str;

use crate::utils::find_project_folder;

use super::model::{Config, Rules, RulesList};

/// Merges a parent configuration with a child configuration
///
/// The child configuration takes precedence over the parent configuration.
/// Rules from both configurations are combined.
///
/// # Arguments
/// * `parent` - The parent configuration
/// * `child` - The child configuration
///
/// # Returns
/// * `Config` - The merged configuration
fn merge_configs(parent: Config, mut child: Config) -> Config {
    // If child has empty root, use parent's root
    if child.root.is_empty() {
        child.root = parent.root;
    }

    // If child has no rules, use parent's rules
    if child.rules.is_empty() {
        child.rules = parent.rules;
    } else {
        // Append parent rules to child rules.
        // Child rules take precedence as they come first in the list
        let mut merged_rules = child.rules;
        merged_rules.extend(parent.rules);
        child.rules = merged_rules;
    }

    // Return the merged configuration
    child
}

/// Loads a configuration from a file
///
/// # Arguments
/// * `file` - Path to the configuration file
///
/// # Returns
/// * `Result<Config>` - The loaded configuration or an error
///
/// # Errors
/// Returns an error if the file cannot be read or if the configuration is invalid
pub fn load_config(file: PathBuf) -> Result<Config> {
    let file_content = fs::read(&file).map_err(|e| {
        anyhow!(
            "Failed to read configuration file {}: {}",
            file.display(),
            e
        )
    })?;

    let content_str = String::from_utf8(file_content).map_err(|e| {
        anyhow!(
            "Configuration file {} contains invalid UTF-8 characters: {}",
            file.display(),
            e
        )
    })?;

    let mut config: Config = from_str(&content_str).map_err(|e| {
        anyhow!(
            "Failed to parse configuration file {}: {}\nPlease check the YAML syntax.",
            file.display(),
            e
        )
    })?;

    // Handle parent configuration if specified
    if let Some(parent_path) = &config.parent {
        debug!("Loading parent configuration from {parent_path}");

        // Resolve parent path relative to the current config file's directory
        let parent_file = if parent_path.starts_with('/') || parent_path.contains(':') {
            // Absolute path
            PathBuf::from(parent_path)
        } else {
            // Relative path - resolve against the directory of the current config file
            let mut parent_file = file.clone();
            parent_file.pop(); // Remove filename to get directory
            parent_file.push(parent_path);
            parent_file
        };

        // Check if parent file exists
        if !parent_file.exists() {
            return Err(anyhow!(
                "Parent configuration file {} specified in {} does not exist",
                parent_file.display(),
                file.display()
            ));
        }

        // Load parent configuration
        let parent_config = load_config(parent_file)?;

        // Merge parent configuration with current configuration
        config = merge_configs(parent_config, config);
    }

    // Validate the merged configuration
    config.validate(true)?;

    Ok(config)
}

/// Loads a configuration from a file without checking path existence
///
/// This is primarily used for testing.
///
/// # Arguments
/// * `file` - Path to the configuration file
///
/// # Returns
/// * `Result<Config>` - The loaded configuration or an error
///
/// # Errors
/// Returns an error if the file cannot be read or if the configuration is invalid
pub fn load_config_for_testing(file: PathBuf) -> Result<Config> {
    let file_content = fs::read(&file).map_err(|e| {
        anyhow!(
            "Failed to read configuration file {}: {}",
            file.display(),
            e
        )
    })?;

    let content_str = String::from_utf8(file_content).map_err(|e| {
        anyhow!(
            "Configuration file {} contains invalid UTF-8 characters: {}",
            file.display(),
            e
        )
    })?;

    let mut config: Config = from_str(&content_str).map_err(|e| {
        anyhow!(
            "Failed to parse configuration file {}: {}\nPlease check the YAML syntax.",
            file.display(),
            e
        )
    })?;

    // Handle parent configuration if specified
    if let Some(parent_path) = &config.parent {
        debug!("Loading parent configuration from {parent_path}");

        // Resolve parent path relative to the current config file's directory
        let parent_file = if parent_path.starts_with('/') || parent_path.contains(':') {
            // Absolute path
            PathBuf::from(parent_path)
        } else {
            // Relative path - resolve against the directory of the current config file
            let mut parent_file = file.clone();
            parent_file.pop(); // Remove filename to get directory
            parent_file.push(parent_path);
            parent_file
        };

        // Check if parent file exists
        if !parent_file.exists() {
            return Err(anyhow!(
                "Parent configuration file {} specified in {} does not exist",
                parent_file.display(),
                file.display()
            ));
        }

        // Load parent configuration
        let parent_config = load_config_for_testing(parent_file)?;

        // Merge parent configuration with current configuration
        config = merge_configs(parent_config, config);
    }

    // Validate the configuration without checking path existence
    config.validate(false)?;

    Ok(config)
}

/// Reads an existing configuration file or creates a new one if it doesn't exist
///
/// # Arguments
/// * `config` - Path to the configuration file
///
/// # Returns
/// * `Result<PathBuf>` - The path to the configuration file or an error
///
/// # Errors
/// Returns an error if the configuration file cannot be created
pub fn read_or_create(config: PathBuf) -> Result<PathBuf> {
    if !&config.exists() {
        create_config_if_not_exists(config)
    } else {
        Ok(config)
    }
}

/// Creates a configuration file in the standard configuration directory if it doesn't exist
///
/// This function determines the appropriate configuration directory for the application
/// based on the platform standards, creates it if necessary, and returns the path to
/// the configuration file within that directory.
///
/// # Arguments
/// * `config` - The base configuration file path
///
/// # Returns
/// * `Result<PathBuf>` - The full path to the configuration file in the standard location
///
/// # Errors
/// * Returns an error if the configuration directory cannot be created
fn create_config_if_not_exists(config: PathBuf) -> Result<PathBuf> {
    let folder = find_project_folder()?;
    Ok(folder.config_dir().join(config))
}

/// Prepares the configuration for execution
///
/// This function processes the patterns in each rule.
///
/// # Arguments
/// * `configuration` - The configuration to prepare
///
/// # Returns
/// * `Result<()>` - Success or an error
///
/// # Errors
/// * Returns an error if pattern processing fails for any rule
pub fn prepare_rules(rules: &mut RulesList) -> Result<()> {
    // Make patterns for each rule
    for mapping in &mut *rules {
        mapping.make_patterns().map_err(|e| {
            error!(
                "Failed to make patterns for rule '{}': {}",
                mapping.title, e
            );
            anyhow!(
                "Failed to make patterns for rule '{}': {}",
                mapping.title,
                e
            )
        })?;
        debug!("Prepared rule: {}", mapping.title);
    }

    info!("Configuration prepared with {} rules", rules.len());
    Ok(())
}

fn add_segment_to_path(segment: &str, path: &mut PathBuf) {
    if segment.starts_with("~") {
        path.push(process_path(segment));
    } else if segment == "." {
        path.push(std::env::current_dir().unwrap());
    } else if segment == ".." {
        path.pop();
    } else {
        path.push(segment);
    }
}

/// Deserialises a value from arrays to a vector of PathBuf
///
/// This function is used to deserialise the root directories field in a Config struct.
/// It can handle both string values and arrays of strings.
pub fn deserialize_from_arrays_to_pathbuf_vec<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<PathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct PathBufVecVisitor;

    impl<'de> serde::de::Visitor<'de> for PathBufVecVisitor {
        type Value = Vec<PathBuf>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("array of arrays of strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut paths = Vec::new();
            while let Some(inner_seq) = seq.next_element::<Vec<String>>()? {
                let mut path = PathBuf::new();
                for segment in inner_seq {
                    add_segment_to_path(&segment, &mut path);
                }
                paths.push(path);
            }
            Ok(paths)
        }
    }

    deserializer.deserialize_seq(PathBufVecVisitor)
}

/// Deserialises a value from an array to an optional PathBuf
///
/// This function is used to deserialise a directory field in a Rule struct.
/// It can handle both string values and arrays of strings.
pub fn deserialize_from_array_to_optional_pathbuf<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<PathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct OptionalPathBufVisitor;

    impl<'de> serde::de::Visitor<'de> for OptionalPathBufVisitor {
        type Value = Option<PathBuf>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or array of strings")
        }

        fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(PathBuf::from(value)))
        }

        fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(self)
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut path = PathBuf::new();
            while let Some(segment) = seq.next_element::<String>()? {
                add_segment_to_path(&segment, &mut path);
            }
            Ok(Some(path))
        }
    }

    deserializer.deserialize_any(OptionalPathBufVisitor)
}

/// Deserialises a value from an array to a PathBuf
///
/// This function is used to deserialise the download directory field in a Config struct.
/// It can handle both string values and arrays of strings.
pub fn deserialize_from_array_to_pathbuf<'de, D>(
    deserializer: D,
) -> std::result::Result<PathBuf, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct PathBufVisitor;

    impl<'de> serde::de::Visitor<'de> for PathBufVisitor {
        type Value = PathBuf;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("array of strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut path = PathBuf::new();
            while let Some(segment) = seq.next_element::<String>()? {
                add_segment_to_path(&segment, &mut path);
            }
            Ok(path)
        }
    }

    deserializer.deserialize_seq(PathBufVisitor)
}

/// Parses rules from a YAML value
///
/// This function is used to deserialise the rules field in a Config struct.
/// It can handle both single rule lists and multiple rule lists.
pub fn parse_rules<'de, D>(deserializer: D) -> std::result::Result<RulesList, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let rules = Rules::deserialize(deserializer)?;
    let mut result = Vec::new();
    match rules {
        Rules::SingleRule(mut rules) => process_rules(&mut rules, &mut result).map_err(|e| {
            error!("Failed to process rules: {e}");
            serde::de::Error::custom(e)
        })?,
        Rules::RootRules(rules) => process_and_append_rules(rules, &mut result).map_err(|e| {
            error!("Failed to process rules: {e}");
            serde::de::Error::custom(e)
        })?,
    };
    result.dedup();
    Ok(result)
}

use crate::config::Rule;
use shellexpand::tilde;

pub fn expand_path(path: &str) -> String {
    tilde(path).to_string()
}

pub fn handle_colon_end(mut path: String) -> String {
    if path.ends_with(':') {
        path += "\\";
    };
    path
}

pub fn process_path<S: AsRef<str>>(path: S) -> String {
    let p = expand_path(path.as_ref());
    handle_colon_end(p)
}

pub fn process_patterns(rule: &mut Rule, patterns: &[String]) -> RulesList {
    patterns
        .iter()
        .map(|pattern| extract_rule_with_pattern(rule, pattern))
        .collect()
}

pub fn map_patterns_to_rules(rule: &mut Rule) -> Result<RulesList> {
    match rule.patterns {
        None => Ok(vec![rule.clone()]),
        Some(ref patterns) => Ok(process_patterns(&mut rule.clone(), patterns)),
    }
}

pub fn extract_rule_with_pattern(rule: &mut Rule, pattern: &str) -> Rule {
    rule.pattern = Some(pattern.to_string());
    let mut derived_rule = rule.clone();
    derived_rule.patterns = None;
    derived_rule
}

pub fn process_and_append_rule(rules: &mut RulesList, new_rules: &mut Vec<Rule>) -> Result<()> {
    for rule in rules {
        let processed_rules = map_patterns_to_rules(rule)?;
        new_rules.extend(processed_rules);
    }
    Ok(())
}

pub fn process_rules(mappings: &mut RulesList, result_mappings: &mut Vec<Rule>) -> Result<()> {
    process_and_append_rule(mappings, result_mappings)
}

pub fn process_and_append_rules(roots: Vec<RulesList>, new_rules: &mut Vec<Rule>) -> Result<()> {
    let roots_with_indices = roots.into_iter().enumerate();
    for (idx, root) in roots_with_indices {
        for mut map in root {
            if map.root == 0 {
                map.root = idx;
            }
            process_and_append_rule(&mut vec![map], new_rules)?;
        }
    }
    Ok(())
}
