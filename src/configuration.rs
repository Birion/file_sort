use std::fs;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::Colorize;
use directories::ProjectDirs;
use glob::glob;
use log::{debug, error, info};
use regex::Regex;
use serde::Deserialize;
use serde_yaml::from_str;

use crate::errors::{generic_error, invalid_filename_error};

use crate::cli::check_for_stdout_stream;
use crate::constants::{APPLICATION, ORGANIZATION, QUALIFIER, WILDCARD};
use crate::logging::format_message;
use crate::parser::*;
use crate::processor::Processor;
use crate::rules::{Rule, RulesList};
use crate::utils::generate_target;

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
        let download_path = self.download.join(WILDCARD);
        let path_str = download_path
            .to_str()
            .ok_or_else(|| invalid_filename_error(download_path.clone()))?;

        for file_path in glob(path_str)? {
            self.files.insert(0, file_path?);
        }
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
        let file_content = fs::read(file)?;
        let content_str = String::from_utf8(file_content)?;
        let config: Config = from_str(&content_str)?;
        Ok(config)
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
        let mut file_processor = Processor::new(file);
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
            processor.target = generate_target(processor, rule, &processor.target)?;
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
        error!("Failed to read the download folder: {}", e);
        anyhow!("Couldn't read the download folder: {}", e)
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
    let folder = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| generic_error("Failed to determine project directories"))?;

    if !folder.config_dir().exists() {
        create_dir_all(folder.config_dir())?;
    }
    Ok(folder.config_dir().join(config))
}
