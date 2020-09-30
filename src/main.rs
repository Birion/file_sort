use std::error::Error;
use std::path::PathBuf;

use clap::ArgMatches;

use crate::lib::get_matches;
use crate::lib::structs::Config;
use crate::lib::config;
use crate::lib::processing::process;

mod lib;

fn main() -> Result<(), Box<dyn Error>> {
    let matches: ArgMatches = get_matches()?;
    let config_file: &str = matches.value_of("config").unwrap();

    let file: PathBuf = config::read(PathBuf::from(config_file))?;

    let mut config: Config = Config::load(file)?;

    let _ = config.get_files().expect("Couldn't read the download folder");

    for mapping in &mut config.mappings {
        let _ = mapping.make_patterns();
    }

    for file in &config.files {
        process(file, &config).unwrap();
    };

    Ok(())
}
