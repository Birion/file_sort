use anyhow::Result;
use atty::Stream;
use clap::{Arg, ArgMatches, command, crate_authors, crate_description, crate_name, crate_version};

pub fn check_for_stdout_stream() {
    if atty::is(Stream::Stdout) {
        dont_disappear::enter_to_continue::default();
    }
}

pub fn get_configuration_file_option() -> Result<ArgMatches> {
    let argument_matches = get_matches()?;
    argument_matches.get_one::<String>("config").unwrap();
    Ok(argument_matches)
}

const CONFIG: &str = "Read from a specific config file";
const DRY: &str = "Run without moving any files";
const DEFAULT_CONFIG_PATH: &str = "config.yaml";

pub fn get_matches() -> Result<ArgMatches> {

    // define arg for reading from specific config file
    let arg_config = Arg::new("config")
        .short('c')
        .long("config")
        .help(CONFIG)
        .default_value(DEFAULT_CONFIG_PATH);

    // define arg for dry run
    let arg_dry = Arg::new("dry")
        .short('n')
        .long("dry")
        .help(DRY)
        .num_args(0);

    let matches = command!()
        .author(crate_authors!())
        .about(crate_description!())
        .name(crate_name!())
        .version(crate_version!())
        .arg(arg_config)
        .arg(arg_dry)
        .get_matches();

    Ok(matches)
}
