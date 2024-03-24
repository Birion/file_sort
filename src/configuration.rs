use std::fs::create_dir_all;
use std::path::PathBuf;

use anyhow::Result;
use clap::ArgMatches;
use directories::ProjectDirs;

use crate::cli::check_for_stdout_stream;
use crate::Config;

pub fn perform_processing_based_on_configuration(argument_matches: ArgMatches) -> Result<()> {
    let configuration_file_path = PathBuf::from(argument_matches.get_one::<String>("config").unwrap());
    let configuration_file = read_or_create(configuration_file_path)?;

    let mut configuration = Config::load(configuration_file)?;
    prepare_configuration(&mut configuration)?;

    execute_based_on_configuration(&configuration, argument_matches.get_flag("dry"))?;

    check_for_stdout_stream();

    Ok(())
}

fn prepare_configuration(configuration: &mut Config) -> Result<()> {
    configuration.get_files().expect("Couldn't read the download folder");

    for mapping in &mut configuration.rules {
        mapping.make_patterns()?;
    }

    Ok(())
}

fn execute_based_on_configuration(configuration: &Config, is_dry_run: bool) -> Result<()> {
    for file in &configuration.files {
        configuration.process(file, is_dry_run)?;
    }

    Ok(())
}

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "Ondřej Vágner";
const APPLICATION: &str = "comic_sort";

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


