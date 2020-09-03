use std::error::Error;
use std::path::PathBuf;

use clap::ArgMatches;

use comic_sort::{get_matches, process_file, read_config};
use comic_sort::structs::Config;

fn main() -> Result<(), Box<dyn Error>> {
    let matches: ArgMatches = get_matches()?;
    let config_file: &str = matches.value_of("config").unwrap();

    let file: PathBuf = read_config(PathBuf::from(config_file))?;

    let mut config: Config = Config::load(file)?;

    let _ = config.get_files().expect("Couldn't read the download folder");

    for mapping in &mut config.mappings {
        let _ = mapping.make_patterns();
    }

    for file in &config.files {
        process_file(file, &config).unwrap();
    };

    Ok(())
}
