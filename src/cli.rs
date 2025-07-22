use atty::Stream;
use clap::{command, crate_authors, crate_description, crate_name, crate_version, Arg, ArgMatches};

use crate::constants::{
    CONFIG_HELP, DEFAULT_CONFIG_PATH, DRY_RUN_HELP, LOCAL_LOGGING_HELP, LOG_FILE_DEFAULT,
    LOG_FILE_HELP, VERBOSE_HELP,
};
use crate::errors::{generic_error, Result};
use crate::logging::LogLevel;
use crate::utils::find_project_folder;

/// Checks if stdout is a terminal and waits for user input if it is
///
/// This function is used to prevent the console window from closing
/// immediately after the program finishes when run from a GUI.
pub fn check_for_stdout_stream() {
    if atty::is(Stream::Stdout) {
        dont_disappear::enter_to_continue::default();
    }
}

/// Gets the configuration file option from command-line arguments
///
/// # Returns
/// * `Result<ArgMatches>` - The command-line arguments with the configuration file option
///
/// # Errors
/// Returns an error if the command-line arguments cannot be parsed
pub fn get_configuration_file_option() -> Result<ArgMatches> {
    let argument_matches = get_matches()?;

    // Verify that the config option exists
    argument_matches
        .get_one::<String>("config")
        .ok_or_else(|| generic_error("Configuration file option not found"))?;

    Ok(argument_matches)
}

/// Sets up and returns command-line argument matches
///
/// Defines the following arguments:
/// - `config`: Path to the configuration file
/// - `dry`: Run without moving any files
/// - `verbose`: Increase verbosity level
///
/// # Returns
/// * `Result<ArgMatches>` - The parsed command-line arguments
///
/// # Errors
/// Returns an error if the command-line arguments cannot be parsed
pub fn get_matches() -> Result<ArgMatches> {
    // define arg for reading from a specific config file
    let arg_config = Arg::new("config")
        .short('c')
        .long("config")
        .help(CONFIG_HELP)
        .default_value(DEFAULT_CONFIG_PATH);

    // define arg for dry run
    let arg_dry = Arg::new("dry")
        .short('n')
        .long("dry")
        .help(DRY_RUN_HELP)
        .num_args(0);

    // define arg for verbosity level
    let arg_verbose = Arg::new("verbose")
        .short('v')
        .long("verbose")
        .help(VERBOSE_HELP)
        .action(clap::ArgAction::Count);

    // define arg for log file
    let log_file = Arg::new("log_file")
        .short('l')
        .long("log-file")
        .help(LOG_FILE_HELP)
        .default_value(LOG_FILE_DEFAULT);

    // define arg for local logging
    let log_locally = Arg::new("log_locally")
        .short('L')
        .long("log-locally")
        .help(LOCAL_LOGGING_HELP)
        .action(clap::ArgAction::SetTrue);

    let matches = command!()
        .author(crate_authors!())
        .about(crate_description!())
        .name(crate_name!())
        .version(crate_version!())
        .arg(arg_config)
        .arg(arg_dry)
        .arg(log_file)
        .arg(log_locally)
        .arg(arg_verbose)
        .get_matches();

    Ok(matches)
}

/// Gets the verbosity level from the command-line arguments
///
/// This function extracts the verbosity level from the command-line arguments
/// by counting the occurrences of the "verbose" flag and converting it to
/// a Verbosity enum value.
///
/// # Arguments
/// * `matches` - The parsed command-line arguments
///
/// # Returns
/// * `Verbosity` - The verbosity level based on the number of -v/--verbose flags
///
/// # Examples
/// ```
/// # use clap::ArgMatches;
/// # use file_sort::cli::get_verbosity;
/// # use file_sort::logging::LogLevel;
/// # fn example(matches: &ArgMatches) {
/// // Get verbosity level from command-line arguments
/// let verbosity = get_verbosity(matches);
///
/// // Use the verbosity level to configure logging
/// match verbosity {
///     LogLevel::Info => println!("Running with normal output"),
///     LogLevel::Debug => println!("Running with debug output"),
///     _ => {}
/// }
/// # }
/// ```
pub fn get_verbosity(matches: &ArgMatches) -> LogLevel {
    let verbose_count = matches.get_count("verbose");
    LogLevel::from_occurrences(verbose_count)
}

pub fn get_log_file(matches: &ArgMatches) -> Result<String> {
    let filename = matches
        .get_one::<String>("log_file")
        .cloned()
        .unwrap_or_else(|| LOG_FILE_DEFAULT.to_string());
    if matches.get_flag("log_locally") {
        Ok(filename)
    } else {
        let folder = find_project_folder()?;
        let path = folder.config_dir().join(filename);
        let path_str = path.as_path().to_str()
            .ok_or_else(|| generic_error(&format!("Failed to convert path to string: {:?}", path)))?;
        Ok(path_str.to_string())
    }
}
