use crate::constants::LOG_FILE_DEFAULT;
use anyhow::Result;
use chrono::SecondsFormat;
use fern::colors::{Color, ColoredLevelConfig};
use fern::Dispatch;
use log::LevelFilter;
use std::str::FromStr;

/// Verbosity level for logging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
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

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "warn" | "warning" => Ok(LogLevel::Warning),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(format!("Unknown verbosity level: {s}")),
        }
    }
}

impl LogLevel {
    /// Convert verbosity level to log::LevelFilter
    pub fn to_level_filter(&self) -> LevelFilter {
        match self {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warning => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }

    /// Get the verbosity level from the number of occurrences of a flag
    pub fn from_occurrences(occurrences: u8) -> Self {
        match occurrences {
            0 => LogLevel::Info,  // Default
            1 => LogLevel::Debug, // -v
            _ => LogLevel::Trace, // -vv or more
        }
    }
}

/// Initialise the logger with the specified verbosity level
pub fn init_logger(verbosity: LogLevel, log_file: &str) -> Result<()> {
    let base_logger = Dispatch::new().level(verbosity.to_level_filter());

    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    let output_logger = Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "\x1B[{}m{}\x1B[0m",
                colors_line.get_color(&record.level()).to_fg_str(),
                message
            ))
        })
        .level(verbosity.to_level_filter())
        .chain(std::io::stdout());

    if !log_file.is_empty() {
        let file_logger = Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{} {} {}] {}",
                    chrono::Local::now().to_rfc3339_opts(SecondsFormat::Secs, true),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .level(verbosity.to_level_filter())
            .chain(fern::log_file(log_file)?);
        base_logger
            .chain(file_logger)
            .chain(output_logger)
            .apply()?;
    } else {
        base_logger.chain(output_logger).apply()?;
    }

    log::debug!("Logger initialized with verbosity level: {verbosity:?}");

    Ok(())
}

/// Initialise the logger with the default verbosity level (Info)
pub fn init_default_logger() -> Result<()> {
    init_logger(LogLevel::Info, LOG_FILE_DEFAULT)
}

/// Format a message with colour support
pub fn format_message(message: &str, colored_message: &str) -> String {
    if atty::is(atty::Stream::Stdout) {
        colored_message.to_string()
    } else {
        message.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        // Test valid log levels
        assert_eq!(LogLevel::from_str("error").unwrap(), LogLevel::Error);
        assert_eq!(LogLevel::from_str("warn").unwrap(), LogLevel::Warning);
        assert_eq!(LogLevel::from_str("warning").unwrap(), LogLevel::Warning);
        assert_eq!(LogLevel::from_str("info").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("debug").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("trace").unwrap(), LogLevel::Trace);

        // Test case insensitivity
        assert_eq!(LogLevel::from_str("ERROR").unwrap(), LogLevel::Error);
        assert_eq!(LogLevel::from_str("Warning").unwrap(), LogLevel::Warning);
        assert_eq!(LogLevel::from_str("Info").unwrap(), LogLevel::Info);

        // Test invalid log level
        let result = LogLevel::from_str("invalid");
        assert!(
            result.is_err(),
            "Should return an error for invalid log level"
        );
        assert_eq!(
            result.unwrap_err(),
            "Unknown verbosity level: invalid",
            "Error message should indicate the unknown level"
        );
    }

    #[test]
    fn test_log_level_to_level_filter() {
        // Test conversion to LevelFilter
        assert_eq!(LogLevel::Error.to_level_filter(), LevelFilter::Error);
        assert_eq!(LogLevel::Warning.to_level_filter(), LevelFilter::Warn);
        assert_eq!(LogLevel::Info.to_level_filter(), LevelFilter::Info);
        assert_eq!(LogLevel::Debug.to_level_filter(), LevelFilter::Debug);
        assert_eq!(LogLevel::Trace.to_level_filter(), LevelFilter::Trace);
    }

    #[test]
    fn test_log_level_from_occurrences() {
        // Test conversion from occurrences
        assert_eq!(LogLevel::from_occurrences(0), LogLevel::Info);
        assert_eq!(LogLevel::from_occurrences(1), LogLevel::Debug);
        assert_eq!(LogLevel::from_occurrences(2), LogLevel::Trace);
        assert_eq!(LogLevel::from_occurrences(3), LogLevel::Trace);
        assert_eq!(LogLevel::from_occurrences(255), LogLevel::Trace);
    }

    #[test]
    fn test_format_message() {
        // Since format_message depends on atty::is which checks if stdout is a terminal,
        // we can't easily test both branches. We'll just test that it returns a string.
        let plain_message = "Test message";
        let colored_message = "\x1B[32mTest message\x1B[0m";

        let result = format_message(plain_message, colored_message);
        assert!(
            result == plain_message || result == colored_message,
            "Result should be either the plain message or the colored message"
        );
    }
}
