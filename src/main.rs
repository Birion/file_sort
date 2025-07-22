use anyhow::Result;
use file_sort::cli::get_log_file;
use file_sort::prelude::*;
use human_panic::setup_panic;

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

    // Process files based on the configuration
    perform_processing_based_on_configuration(matches)
}
