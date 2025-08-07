use atty::Stream;
use clap::{Arg, ArgMatches, Command, command, crate_authors, crate_description, crate_name, crate_version};

use crate::constants::{
    CONFIG_HELP, DEFAULT_CONFIG_PATH, DRY_RUN_HELP, LOCAL_LOGGING_HELP, LOG_FILE_DEFAULT,
    LOG_FILE_HELP, VERBOSE_HELP, WIZARD_HELP, WIZARD_OUTPUT_HELP,
};
use crate::errors::{Result, generic_error};
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

    // Create the wizard subcommand
    let wizard_cmd = Command::new("wizard")
        .about(WIZARD_HELP)
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help(WIZARD_OUTPUT_HELP)
                .default_value(DEFAULT_CONFIG_PATH)
        );

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
        .subcommand(wizard_cmd)
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
        let path_str = path
            .as_path()
            .to_str()
            .ok_or_else(|| generic_error(&format!("Failed to convert path to string: {path:?}")))?;
        Ok(path_str.to_string())
    }
}

/// Checks if the wizard subcommand was used
///
/// # Arguments
/// * `matches` - The parsed command-line arguments
///
/// # Returns
/// * `bool` - True if the wizard subcommand was used, false otherwise
pub fn is_wizard_command(matches: &ArgMatches) -> bool {
    matches.subcommand_matches("wizard").is_some()
}

/// Gets the output path for the wizard command
///
/// # Arguments
/// * `matches` - The parsed command-line arguments
///
/// # Returns
/// * `Result<String>` - The output path or an error
///
/// # Errors
/// Returns an error if the output path cannot be determined
pub fn get_wizard_output_path(matches: &ArgMatches) -> Result<String> {
    if let Some(wizard_matches) = matches.subcommand_matches("wizard") {
        let output = wizard_matches
            .get_one::<String>("output")
            .cloned()
            .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());
        Ok(output)
    } else {
        Err(generic_error("Wizard command not found"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use predicates::prelude::*;

    fn make_test_command_logging() -> clap::Command {
        clap::Command::new("fsort")
            .arg(
                Arg::new("log_file")
                    .short('l')
                    .long("log-file")
                    .default_value(LOG_FILE_DEFAULT),
            )
            .arg(
                Arg::new("log_locally")
                    .short('L')
                    .long("log-locally")
                    .action(clap::ArgAction::SetTrue),
            )
    }

    fn make_test_command_verbosity() -> clap::Command {
        clap::Command::new("fsort").arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count),
        )
    }

    #[test]
    fn test_get_configuration_file_option_failure() {
        let mut cmd = Command::cargo_bin("fsort").expect("Failed to create command");
        let assert = cmd.arg("--config").arg("test_config.toml").assert();

        assert.failure();
    }

    #[test]
    fn test_get_configuration_file_option_default() {
        let mut cmd = Command::cargo_bin("fsort").expect("Failed to create command");
        let assert = cmd.arg("--help").assert();

        assert
            .success()
            .stdout(predicate::str::contains(DEFAULT_CONFIG_PATH));
    }

    #[test]
    fn test_get_matches_config_option() {
        let mut cmd = Command::cargo_bin("fsort").expect("Failed to create command");
        let assert = cmd.arg("--help").assert();

        assert
            .success()
            .stdout(predicate::str::contains("-c, --config"))
            .stdout(predicate::str::contains("Read from a specific config file"));
    }

    #[test]
    fn test_get_matches_dry_run_option() {
        let mut cmd = Command::cargo_bin("fsort").expect("Failed to create command");
        let assert = cmd.arg("--help").assert();

        assert
            .success()
            .stdout(predicate::str::contains("-n, --dry"))
            .stdout(predicate::str::contains("Run without moving any files"));
    }

    #[test]
    fn test_get_matches_verbose_option() {
        let mut cmd = Command::cargo_bin("fsort").expect("Failed to create command");
        let assert = cmd.arg("--help").assert();

        assert
            .success()
            .stdout(predicate::str::contains("-v, --verbose"))
            .stdout(predicate::str::contains("Increase verbosity level"));
    }

    #[test]
    fn test_get_matches_log_file_option() {
        let mut cmd = Command::cargo_bin("fsort").expect("Failed to create command");
        let assert = cmd.arg("--help").assert();

        assert
            .success()
            .stdout(predicate::str::contains("-l, --log-file"))
            .stdout(predicate::str::contains("Path to the log file"));
    }

    #[test]
    fn test_get_matches_log_locally_option() {
        let mut cmd = Command::cargo_bin("fsort").expect("Failed to create command");
        let assert = cmd.arg("--help").assert();

        assert
            .success()
            .stdout(predicate::str::contains("-L, --log-locally"))
            .stdout(predicate::str::contains(
                "Log messages in the current directory",
            ));
    }

    #[test]
    fn test_get_verbosity_default() {
        // This is a unit test for the get_verbosity function
        // We need to create a mock ArgMatches with a verbose count of 0
        let matches = make_test_command_verbosity().get_matches_from(vec!["fsort"]);

        let verbosity = get_verbosity(&matches);
        assert_eq!(verbosity, LogLevel::Info);
    }

    #[test]
    fn test_get_verbosity_debug() {
        // Test with one verbose flag
        let matches = make_test_command_verbosity().get_matches_from(vec!["fsort", "-v"]);

        let verbosity = get_verbosity(&matches);
        assert_eq!(verbosity, LogLevel::Debug);
    }

    #[test]
    fn test_get_verbosity_trace() {
        // Test with two verbose flags
        let matches = make_test_command_verbosity().get_matches_from(vec!["fsort", "-v", "-v"]);

        let verbosity = get_verbosity(&matches);
        assert_eq!(verbosity, LogLevel::Trace);
    }

    #[test]
    fn test_get_log_file_default() {
        // Test with default log file
        let matches = make_test_command_logging().get_matches_from(vec!["fsort"]);

        // Since log_locally is not set, this should return a path in the project folder
        // which we can't easily test, so we'll just check that it contains the default log file name
        let log_file = get_log_file(&matches).expect("Failed to get log file");
        assert!(log_file.contains(LOG_FILE_DEFAULT));
    }

    #[test]
    fn test_get_log_file_custom() {
        // Test with custom log file
        let custom_log = "custom.log";
        let matches =
            make_test_command_logging().get_matches_from(vec!["fsort", "--log-file", custom_log]);

        // Since log_locally is not set, this should return a path in the project folder
        let log_file = get_log_file(&matches).expect("Failed to get log file");
        assert!(log_file.contains(custom_log));
    }

    #[test]
    fn test_get_log_file_local() {
        // Test with local logging
        let matches = make_test_command_logging().get_matches_from(vec!["fsort", "--log-locally"]);

        // With log_locally set, this should return just the log file name
        let log_file = get_log_file(&matches).expect("Failed to get log file");
        assert_eq!(log_file, LOG_FILE_DEFAULT);
    }

    #[test]
    fn test_get_log_file_local_custom() {
        // Test with local logging and custom log file
        let custom_log = "custom.log";
        let matches = make_test_command_logging().get_matches_from(vec![
            "fsort",
            "--log-locally",
            "--log-file",
            custom_log,
        ]);

        // With log_locally set, this should return just the custom log file name
        let log_file = get_log_file(&matches).expect("Failed to get log file");
        assert_eq!(log_file, custom_log);
    }

    // Note: We're not testing check_for_stdout_stream() as it involves user interaction
    // which is difficult to test in an automated way
}
