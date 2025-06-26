use env_logger::{Builder, Env};
use log::LevelFilter;
use std::str::FromStr;

/// Verbosity level for logging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    /// Error messages only
    Error,
    /// Warning and error messages
    Warning,
    /// Info, warning, and error messages (default)
    Info,
    /// Debug, info, warning, and error messages
    Debug,
    /// Trace, debug, info, warning, and error messages
    Trace,
}

impl FromStr for Verbosity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Verbosity::Error),
            "warn" | "warning" => Ok(Verbosity::Warning),
            "info" => Ok(Verbosity::Info),
            "debug" => Ok(Verbosity::Debug),
            "trace" => Ok(Verbosity::Trace),
            _ => Err(format!("Unknown verbosity level: {}", s)),
        }
    }
}

impl Verbosity {
    /// Convert verbosity level to log::LevelFilter
    pub fn to_level_filter(&self) -> LevelFilter {
        match self {
            Verbosity::Error => LevelFilter::Error,
            Verbosity::Warning => LevelFilter::Warn,
            Verbosity::Info => LevelFilter::Info,
            Verbosity::Debug => LevelFilter::Debug,
            Verbosity::Trace => LevelFilter::Trace,
        }
    }

    /// Get the verbosity level from the number of occurrences of a flag
    pub fn from_occurrences(occurrences: u8) -> Self {
        match occurrences {
            0 => Verbosity::Info,  // Default
            1 => Verbosity::Debug, // -v
            _ => Verbosity::Trace, // -vv or more
        }
    }
}

/// Initialise the logger with the specified verbosity level
pub fn init_logger(verbosity: Verbosity) {
    let env = Env::default().filter_or("FSORT_LOG_LEVEL", verbosity.to_level_filter().to_string());

    Builder::from_env(env)
        .format_timestamp(None) // No timestamp in the output
        .format_module_path(false) // No module path in the output
        .init();

    log::debug!("Logger initialized with verbosity level: {:?}", verbosity);
}

/// Initialise the logger with the default verbosity level (Info)
pub fn init_default_logger() {
    init_logger(Verbosity::Info);
}

/// Format a message with colour support
pub fn format_message(message: &str, colored_message: &str) -> String {
    if atty::is(atty::Stream::Stdout) {
        colored_message.to_string()
    } else {
        message.to_string()
    }
}
