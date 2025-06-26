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
            _ => Err(format!("Unknown verbosity level: {}", s)),
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
pub fn init_logger(verbosity: LogLevel) -> Result<()> {
    let base_logger = Dispatch::new().level(verbosity.to_level_filter());

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
        .chain(fern::log_file("fsort.log")?);

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

    base_logger
        .chain(file_logger)
        .chain(output_logger)
        .apply()?;

    log::debug!("Logger initialized with verbosity level: {:?}", verbosity);

    Ok(())
}

/// Initialise the logger with the default verbosity level (Info)
pub fn init_default_logger() -> Result<()> {
    init_logger(LogLevel::Info)
}

/// Format a message with colour support
pub fn format_message(message: &str, colored_message: &str) -> String {
    if atty::is(atty::Stream::Stdout) {
        colored_message.to_string()
    } else {
        message.to_string()
    }
}
