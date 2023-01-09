use std::error::Error;
use std::path::PathBuf;

use clap::ArgMatches;
use human_panic::setup_panic;
use comic_sort::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    setup_panic!();
    let matches: ArgMatches = get_matches()?;
    let config_file: &str = matches.value_of("config").unwrap();

    let file: PathBuf = read(PathBuf::from(config_file))?;

    let mut config: Config = Config::load(file)?;

    config.get_files().expect("Couldn't read the download folder");

    for mapping in &mut config.mappings {
        let _ = mapping.make_patterns();
    }

    for file in &config.files {
        let _ = config.process(file);
    };

    Ok(())
}
