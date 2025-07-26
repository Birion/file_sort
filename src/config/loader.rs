//! Configuration loading functionality
//!
//! This module contains functions for loading and validating configuration.

use std::fs;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use log::{debug, error, info};
use serde::Deserialize;
use serde_yaml::from_str;

use crate::utils::find_project_folder;

use super::model::{Config, Rules, RulesList};

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

    let config: Config = from_str(&content_str).map_err(|e| {
        anyhow!(
            "Failed to parse configuration file {}: {}\nPlease check the YAML syntax.",
            file.display(),
            e
        )
    })?;

    // Validate the configuration
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

    let config: Config = from_str(&content_str).map_err(|e| {
        anyhow!(
            "Failed to parse configuration file {}: {}\nPlease check the YAML syntax.",
            file.display(),
            e
        )
    })?;

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

/// Deserializes a value from arrays to a vector of PathBuf
///
/// This function is used to deserialize the root directories field in a Config struct.
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
                    path.push(segment);
                }
                paths.push(path);
            }
            Ok(paths)
        }
    }

    deserializer.deserialize_seq(PathBufVecVisitor)
}

/// Deserializes a value from an array to a PathBuf
///
/// This function is used to deserialize the download directory field in a Config struct.
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
                path.push(segment);
            }
            Ok(path)
        }
    }

    deserializer.deserialize_seq(PathBufVisitor)
}

/// Parses rules from a YAML value
///
/// This function is used to deserialize the rules field in a Config struct.
/// It can handle both single rule lists and multiple rule lists.
pub fn parse_rules<'de, D>(deserializer: D) -> std::result::Result<RulesList, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let rules = Rules::deserialize(deserializer)?;
    match rules {
        Rules::SingleRule(rules) => Ok(rules),
        Rules::RootRules(rules) => {
            if rules.is_empty() {
                Ok(Vec::new())
            } else {
                Ok(rules[0].clone())
            }
        }
    }
}
