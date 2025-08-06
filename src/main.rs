use anyhow::Result;
use file_sort::cli::{
    check_for_stdout_stream, get_configuration_file_option, get_log_file, get_verbosity,
};
use file_sort::logging::init_logger;
use file_sort::workflow::{process_files, ProcessingOptions};
use human_panic::setup_panic;
use std::path::PathBuf;

fn main() -> Result<()> {
    setup_panic!();

    // Get command-line arguments
    let matches = get_configuration_file_option()?;

    // Initialise the logger with the verbosity level from the command-line arguments
    let verbosity = get_verbosity(&matches);
    let log_file = if matches.get_flag("dry") {
        String::default()
    } else {
        get_log_file(&matches)?
    };
    init_logger(verbosity, &log_file)?;

    // Get the configuration file path and dry run flag
    let config_path = PathBuf::from(matches.get_one::<String>("config").unwrap());
    let dry_run = matches.get_flag("dry");

    // Process files based on the configuration
    let options = ProcessingOptions {
        config_path,
        dry_run,
    };

    process_files(options)?;

    check_for_stdout_stream();

    Ok(())
}
