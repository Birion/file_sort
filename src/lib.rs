pub mod cli;
pub mod configuration;
pub mod constants;
pub mod errors;
pub mod folder_function;
pub mod logging;
pub mod parser;
pub mod processor;
pub mod rules;
pub mod utils;

/// Prelude module that re-exports commonly used items
///
/// This module provides a convenient way to import all the commonly used
/// types, functions, and error helpers with a single import statement.
pub mod prelude {
    pub use crate::cli::{get_configuration_file_option, get_verbosity};
    pub use crate::configuration::perform_processing_based_on_configuration;
    pub use crate::errors::{
        config_parsing_error, directory_not_found_error, file_operation_error, generic_error,
        glob_pattern_error, invalid_filename_error, no_match_error, path_operation_error,
        pattern_extraction_error, pattern_matching_error,
    };
    pub use crate::errors::{Error, Result};
    pub use crate::logging::{format_message, init_default_logger, init_logger, LogLevel};
}
