use std::path::PathBuf;

use atty::Stream;
use anyhow::Result;
use clap::ArgMatches;
use human_panic::setup_panic;
use comic_sort::prelude::*;

fn main() -> Result<()> {
    setup_panic!();
    let matches: ArgMatches = get_matches()?;
    let config_file: &str = matches.get_one::<String>("config").unwrap();

    let file: PathBuf = read(PathBuf::from(config_file))?;

    let mut config: Config = Config::load(file)?;

    config.get_files().expect("Couldn't read the download folder");

    for mapping in &mut config.mappings {
        let _ = mapping.make_patterns();
    }

    for file in &config.files {
        let _ = config.process(file);
    };

    if atty::is(Stream::Stdout) {
        dont_disappear::enter_to_continue::default();
    }

    Ok(())
}
