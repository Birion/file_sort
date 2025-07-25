use std::fs;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::Colorize;
use log::{debug, error, info};
use regex::Regex;
use serde::Deserialize;
use serde_yaml::from_str;

use crate::errors::generic_error;

use crate::cli::check_for_stdout_stream;
use crate::logging::format_message;
use crate::parser::*;
use crate::processor::Processor;
use crate::rules::{Rule, RulesList};
use crate::utils::{find_project_folder, generate_target, is_hidden_file};

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
}

impl Config {
    /// Gets all files from the download directory and stores them in the files vector
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error
    ///
    /// # Errors
    /// Returns an error if the download directory cannot be read or if a file path is invalid
    pub fn get_files(&mut self) -> Result<()> {
        self.files = read_dir(&self.download)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| !is_hidden_file(path))
            .filter(|path| path.is_file())
            .collect();
        Ok(())
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
    pub fn load(file: PathBuf) -> Result<Config> {
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
    pub fn load_for_testing(file: PathBuf) -> Result<Config> {
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
                    debug!(
                        "Warning: Directory for rule '{}' does not exist: {}",
                        rule.title,
                        full_path.display()
                    );
                }
            }
        }

        Ok(())
    }

    /// Processes a file according to the rules in the configuration
    ///
    /// # Arguments
    /// * `file` - Path to the file to process
    /// * `run_execution` - Whether to actually perform the file operations (false) or just simulate them (true)
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error
    ///
    /// # Errors
    /// Returns an error if the file cannot be processed
    pub fn process(&self, file: &Path, run_execution: bool) -> Result<()> {
        // Create a processor using the builder pattern
        let mut file_processor = Processor::builder(file).build();

        for rule in &self.rules {
            if let Ok(applied_rule) = self.apply_rule(rule, &mut file_processor) {
                let source_filename = applied_rule.source_filename()?;
                let title = &rule.title;

                // Log the file found and rule being applied
                let message = format!("{source_filename} found! Applying setup for {title}.");
                let colored_message = format!(
                    "{} found! Applying setup for {}.",
                    source_filename.bold(),
                    title.bold().blue()
                );
                info!("{}", format_message(&message, &colored_message));

                // Log the new filename if changed
                if applied_rule.is_changed()? {
                    let target_filename = applied_rule.target_filename()?;
                    let message = format!("New filename: {target_filename}");
                    let colored_message = format!("New filename: {}", target_filename.bold().red());
                    info!("{}", format_message(&message, &colored_message));
                }

                // Add a blank line for readability in logs
                debug!("");

                // Perform the file action if not a dry run
                if !run_execution {
                    applied_rule.perform_file_action(rule.copy)?;
                }
            }
        }

        Ok(())
    }

    /// Applies a rule to a file processor
    ///
    /// # Arguments
    /// * `rule` - The rule to apply
    /// * `processor` - The file processor to apply the rule to
    ///
    /// # Returns
    /// * `Result<Processor>` - The updated processor or an error
    ///
    /// # Errors
    /// Returns an error if the rule cannot be applied to the file
    fn apply_rule(&self, rule: &Rule, processor: &mut Processor) -> Result<Processor> {
        let root_path = &self.root[rule.root];
        let pattern = Regex::new(rule.old_pattern.as_str())?;
        if pattern.is_match(processor.source_filename()?) {
            let directory = match &rule.directory {
                None => PathBuf::from(&rule.title),
                Some(dir) => dir.to_owned(),
            };
            processor.create_and_set_target_directory(root_path, &directory)?;
            let target = generate_target(processor, rule, processor.target())?;
            processor.set_target(target);
            Ok(processor.to_owned())
        } else {
            Err(anyhow!("Pattern doesn't match."))
        }
    }
}

/// Performs file processing based on the provided command-line arguments
///
/// This is the main entry point for the file sorting functionality.
///
/// # Arguments
/// * `argument_matches` - The parsed command-line arguments
///
/// # Returns
/// * `Result<()>` - Success or an error
///
/// # Errors
/// Returns an error if the configuration cannot be loaded or if file processing fails
pub fn perform_processing_based_on_configuration(argument_matches: ArgMatches) -> Result<()> {
    let config_arg = argument_matches
        .get_one::<String>("config")
        .ok_or_else(|| generic_error("Configuration file path not provided"))?;
    let configuration_file_path = PathBuf::from(config_arg);
    let configuration_file = read_or_create(configuration_file_path)?;

    let mut configuration = Config::load(configuration_file)?;
    prepare_configuration(&mut configuration)?;

    execute_based_on_configuration(&configuration, argument_matches.get_flag("dry"))?;

    check_for_stdout_stream();

    Ok(())
}

/// Prepares the configuration for execution
///
/// This function performs two main tasks:
/// 1. Gets all files from the download directory
/// 2. Processes the patterns in each rule
///
/// # Arguments
/// * `configuration` - The configuration to prepare
///
/// # Returns
/// * `Result<()>` - Success or an error
///
/// # Errors
/// * Returns an error if the download directory cannot be read
/// * Returns an error if pattern processing fails for any rule
fn prepare_configuration(configuration: &mut Config) -> Result<()> {
    // Get files from the download folder
    configuration.get_files().map_err(|e| {
        error!("Failed to read the download folder: {e}");
        anyhow!("Couldn't read the download folder: {e}")
    })?;

    debug!(
        "Found {} files in the download folder",
        configuration.files.len()
    );

    // Make patterns for each rule
    for mapping in &mut configuration.rules {
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

    info!(
        "Configuration prepared with {} rules",
        configuration.rules.len()
    );
    Ok(())
}

/// Executes file processing based on the configuration
///
/// This function processes all files in the configuration according to the rules.
/// It can operate in either normal mode or dry-run mode.
///
/// # Arguments
/// * `configuration` - The configuration containing rules and files to process
/// * `is_dry_run` - If true, no actual file operations will be performed
///
/// # Returns
/// * `Result<()>` - Success or an error
///
/// # Errors
/// * Returns an error if any file processing operation fails
fn execute_based_on_configuration(configuration: &Config, is_dry_run: bool) -> Result<()> {
    let file_count = configuration.files.len();

    if file_count == 0 {
        info!("No files found in the download folder");
        return Ok(());
    }

    info!(
        "Processing {} files{}...",
        file_count,
        if is_dry_run { " (dry run)" } else { "" }
    );

    for (index, file) in configuration.files.iter().enumerate() {
        debug!("Processing file {}/{}: {:?}", index + 1, file_count, file);
        configuration.process(file, is_dry_run).map_err(|e| {
            error!("Failed to process file {file:?}: {e}");
            anyhow!("Failed to process file {file:?}: {e}")
        })?;
    }

    info!("Finished processing {file_count} files");
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Helper function to create a test configuration file and parse it
    fn parse_test_config(config_content: &str) -> Result<Config> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("test_config.yaml");

        fs::write(&config_path, config_content)?;

        // Use the special load_for_testing method that doesn't check path existence
        Config::load_for_testing(config_path)
    }

    #[test]
    fn test_valid_configuration() {
        // A minimal valid configuration
        let valid_config = r#"
root:
  - - "~"
    - "Documents"
download:
  - "~"
  - "Downloads"
rules:
  - title: "Test Rule"
    pattern: "test.txt"
"#;

        let result = parse_test_config(valid_config);
        assert!(
            result.is_ok(),
            "Valid configuration failed validation: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_missing_root_directories() {
        let invalid_config = r#"
download:
  - "~"
  - "Downloads"
rules:
  - title: "Test Rule"
    pattern: "test.txt"
"#;

        let result = parse_test_config(invalid_config);
        assert!(
            result.is_err(),
            "Configuration with missing root directories should fail validation"
        );

        let error = result.err().unwrap();
        assert!(
            error.to_string().contains("missing field `root`"),
            "Error message should mention missing root field: {error}"
        );
    }

    #[test]
    fn test_empty_root_directories() {
        let invalid_config = r#"
root: []
download:
  - "~"
  - "Downloads"
rules:
  - title: "Test Rule"
    pattern: "test.txt"
"#;

        let result = parse_test_config(invalid_config);
        assert!(
            result.is_err(),
            "Configuration with empty root directories should fail validation"
        );

        let error = result.err().unwrap();
        assert!(
            error.to_string().contains("No root directories specified"),
            "Error message should mention no root directories: {error}"
        );
    }

    #[test]
    fn test_missing_download_directory() {
        let invalid_config = r#"
root:
  - - "~"
    - "Documents"
rules:
  - title: "Test Rule"
    pattern: "test.txt"
"#;

        let result = parse_test_config(invalid_config);
        assert!(
            result.is_err(),
            "Configuration with missing download directory should fail validation"
        );

        // The error will be from serde deserialization since download is a required field
        let error = result.err().unwrap();
        assert!(
            error.to_string().contains("missing field `download`"),
            "Error message should mention missing download directory: {error}"
        );
    }

    #[test]
    fn test_missing_rules() {
        let invalid_config = r#"
root:
  - - "~"
    - "Documents"
download:
  - "~"
  - "Downloads"
rules: []
"#;

        let result = parse_test_config(invalid_config);
        assert!(
            result.is_err(),
            "Configuration with empty rules should fail validation"
        );

        let error = result.err().unwrap();
        assert!(
            error.to_string().contains("No rules specified"),
            "Error message should mention missing rules: {error}"
        );
    }

    #[test]
    fn test_rule_without_title() {
        let invalid_config = r#"
root:
  - - "~"
    - "Documents"
download:
  - "~"
  - "Downloads"
rules:
  - title: ""
    pattern: "test.txt"
"#;

        let result = parse_test_config(invalid_config);
        assert!(
            result.is_err(),
            "Configuration with empty rule title should fail validation"
        );

        let error = result.err().unwrap();
        assert!(
            error.to_string().contains("empty title"),
            "Error message should mention empty title: {error}"
        );
    }

    #[test]
    fn test_rule_without_pattern() {
        let invalid_config = r#"
root:
  - - "~"
    - "Documents"
download:
  - "~"
  - "Downloads"
rules:
  - title: "Test Rule"
"#;

        let result = parse_test_config(invalid_config);
        assert!(
            result.is_err(),
            "Configuration with missing pattern should fail validation"
        );

        let error = result.err().unwrap();
        assert!(
            error
                .to_string()
                .contains("no pattern or patterns specified"),
            "Error message should mention missing pattern: {error}"
        );
    }

    #[test]
    fn test_rule_with_invalid_root_index() {
        let invalid_config = r#"
root:
  - - "~"
    - "Documents"
download:
  - "~"
  - "Downloads"
rules:
  - title: "Test Rule"
    pattern: "test.txt"
    root: 5
"#;

        let result = parse_test_config(invalid_config);
        assert!(
            result.is_err(),
            "Configuration with invalid root index should fail validation"
        );

        let error = result.err().unwrap();
        assert!(
            error.to_string().contains("out of bounds"),
            "Error message should mention out of bounds root index: {error}"
        );
    }
}
