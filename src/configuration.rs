use std::fs;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::Colorize;
use directories::ProjectDirs;
use glob::glob;
use log::{debug, error, info, warn};
use regex::Regex;
use serde::Deserialize;
use serde_yaml::from_str;

use crate::cli::check_for_stdout_stream;
use crate::logging::format_message;
use crate::parser::*;
use crate::utils::generate_target;
use crate::{Processor, Rule, RulesList, APPLICATION, ORGANIZATION, QUALIFIER, WILDCARD};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_from_arrays_to_pathbuf_vec")]
    pub root: Vec<PathBuf>,
    #[serde(deserialize_with = "deserialize_from_array_to_pathbuf")]
    pub download: PathBuf,
    #[serde(deserialize_with = "parse_rules")]
    pub rules: RulesList,
    #[serde(skip_deserializing)]
    pub files: Vec<PathBuf>,
}

impl Config {
    pub fn get_files(&mut self) -> Result<()> {
        for file_path in glob(self.download.join(WILDCARD).to_str().unwrap())? {
            self.files.insert(0, file_path?);
        }
        Ok(())
    }

    pub fn load(file: PathBuf) -> Result<Config> {
        let file_content = fs::read(file)?;
        let content_str = String::from_utf8(file_content)?;
        let config: Config = from_str(&content_str)?;
        Ok(config)
    }

    pub fn process(&self, file: &Path, run_execution: bool) -> Result<()> {
        let mut file_processor = Processor::new(file);
        for rule in &self.rules {
            if let Ok(applied_rule) = self.apply_rule(rule, &mut file_processor) {
                let source_filename = applied_rule.source_filename()?;
                let title = &rule.title;

                // Log the file found and rule being applied
                let message = format!("{} found! Applying setup for {}.", source_filename, title);
                let colored_message = format!(
                    "{} found! Applying setup for {}.",
                    source_filename.bold(),
                    title.bold().blue()
                );
                info!("{}", format_message(&message, &colored_message));

                // Log the new filename if changed
                if applied_rule.is_changed()? {
                    let target_filename = applied_rule.target_filename()?;
                    let message = format!("New filename: {}", target_filename);
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

pub fn perform_processing_based_on_configuration(argument_matches: ArgMatches) -> Result<()> {
    let configuration_file_path =
        PathBuf::from(argument_matches.get_one::<String>("config").unwrap());
    let configuration_file = read_or_create(configuration_file_path)?;

    let mut configuration = Config::load(configuration_file)?;
    prepare_configuration(&mut configuration)?;

    execute_based_on_configuration(&configuration, argument_matches.get_flag("dry"))?;

    check_for_stdout_stream();

    Ok(())
}

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
            error!("Failed to process file {:?}: {}", file, e);
            anyhow!("Failed to process file {:?}: {}", file, e)
        })?;
    }

    info!("Finished processing {} files", file_count);
    Ok(())
}

pub fn read_or_create(config: PathBuf) -> Result<PathBuf> {
    if !&config.exists() {
        create_config_if_not_exists(config)
    } else {
        Ok(config)
    }
}

fn create_config_if_not_exists(config: PathBuf) -> Result<PathBuf> {
    let folder = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).unwrap();
    if !folder.config_dir().exists() {
        create_dir_all(folder.config_dir())?;
    }
    Ok(folder.config_dir().join(config))
}
