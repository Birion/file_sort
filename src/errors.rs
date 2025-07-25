use glob::PatternError;
use regex::Error as RegexError;
use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// Custom error type for the File Sort application
#[derive(Debug)]
pub enum Error {
    /// Error related to file operations
    FileOperation {
        source: io::Error,
        path: PathBuf,
        operation: String,
    },
    /// Error related to pattern matching
    PatternMatching { source: RegexError, pattern: String },
    /// Error related to glob pattern matching
    GlobPattern {
        source: PatternError,
        pattern: String,
    },
    /// Error related to path operations
    PathOperation { path: PathBuf, operation: String },
    /// Error related to configuration parsing
    ConfigParsing {
        source: Box<dyn StdError + Send + Sync>,
        detail: String,
    },
    /// Error related to pattern extraction
    PatternExtraction { pattern: String, detail: String },
    /// Error when no match is found
    NoMatch { pattern: String, value: String },
    /// Error when a filename is not valid Unicode
    InvalidFilename { path: PathBuf },
    /// Error when a directory is not found
    DirectoryNotFound { path: PathBuf },
    /// Generic error with a message
    Generic { message: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FileOperation {
                path, operation, ..
            } => {
                write!(f, "Failed to {} file: {}", operation, path.display())
            }
            Error::PatternMatching { pattern, .. } => {
                write!(f, "Invalid pattern: {pattern}")
            }
            Error::GlobPattern { pattern, .. } => {
                write!(f, "Invalid glob pattern: {pattern}")
            }
            Error::PathOperation { path, operation } => {
                write!(f, "Failed to {} path: {}", operation, path.display())
            }
            Error::ConfigParsing { detail, .. } => {
                write!(f, "Configuration parsing error: {detail}")
            }
            Error::PatternExtraction { pattern, detail } => {
                write!(f, "Failed to extract pattern from '{pattern}': {detail}")
            }
            Error::NoMatch { pattern, value } => {
                write!(f, "No match found for pattern '{pattern}' in '{value}'")
            }
            Error::InvalidFilename { path } => {
                write!(f, "Filename is not valid unicode: {}", path.display())
            }
            Error::DirectoryNotFound { path } => {
                write!(f, "Directory not found: {}", path.display())
            }
            Error::Generic { message } => {
                write!(f, "{message}")
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::FileOperation { source, .. } => Some(source),
            Error::PatternMatching { source, .. } => Some(source),
            Error::GlobPattern { source, .. } => Some(source),
            Error::ConfigParsing { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::FileOperation {
            source: err,
            path: PathBuf::new(),
            operation: "perform operation on".to_string(),
        }
    }
}

impl From<RegexError> for Error {
    fn from(err: RegexError) -> Self {
        Error::PatternMatching {
            source: err,
            pattern: String::new(),
        }
    }
}

impl From<PatternError> for Error {
    fn from(err: PatternError) -> Self {
        Error::GlobPattern {
            source: err,
            pattern: String::new(),
        }
    }
}

/// Custom Result type for the File Sort application
///
/// This type alias simplifies error handling throughout the application by
/// using the custom Error type. It's used as the return type for most functions
/// that can fail.
///
/// # Type Parameters
/// * `T` - The type of the successful result
///
/// # Examples
/// ```
/// use file_sort::prelude::{Result, generic_error};
///
/// fn example_function() -> Result<String> {
///     // Return success
///     Ok("success".to_string())
///     
///     // Or return an error
///     // Err(generic_error("Something went wrong"))
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

/// Helper function to create a file operation error
pub fn file_operation_error(err: io::Error, path: PathBuf, operation: &str) -> Error {
    Error::FileOperation {
        source: err,
        path,
        operation: operation.to_string(),
    }
}

/// Helper function to create a pattern matching error
pub fn pattern_matching_error(err: RegexError, pattern: &str) -> Error {
    Error::PatternMatching {
        source: err,
        pattern: pattern.to_string(),
    }
}

/// Helper function to create a glob pattern error
pub fn glob_pattern_error(err: PatternError, pattern: &str) -> Error {
    Error::GlobPattern {
        source: err,
        pattern: pattern.to_string(),
    }
}

/// Helper function to create a path operation error
pub fn path_operation_error(path: PathBuf, operation: &str) -> Error {
    Error::PathOperation {
        path,
        operation: operation.to_string(),
    }
}

/// Helper function to create a config parsing error
pub fn config_parsing_error<E: StdError + Send + Sync + 'static>(err: E, detail: &str) -> Error {
    Error::ConfigParsing {
        source: Box::new(err),
        detail: detail.to_string(),
    }
}

/// Helper function to create a pattern extraction error
pub fn pattern_extraction_error(pattern: &str, detail: &str) -> Error {
    Error::PatternExtraction {
        pattern: pattern.to_string(),
        detail: detail.to_string(),
    }
}

/// Helper function to create a no-match error
pub fn no_match_error(pattern: &str, value: &str) -> Error {
    Error::NoMatch {
        pattern: pattern.to_string(),
        value: value.to_string(),
    }
}

/// Helper function to create an invalid filename error
pub fn invalid_filename_error(path: PathBuf) -> Error {
    Error::InvalidFilename { path }
}

/// Helper function to create a directory not found error
pub fn directory_not_found_error(path: PathBuf) -> Error {
    Error::DirectoryNotFound { path }
}

/// Helper function to create a generic error
pub fn generic_error(message: &str) -> Error {
    Error::Generic {
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_operation_error() {
        let path = PathBuf::from("/test/path");
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = file_operation_error(io_error, path.clone(), "read");

        // Check that the error contains the expected information
        let error_string = format!("{error}");
        assert!(
            error_string.contains("read"),
            "Error message should contain the operation"
        );
        assert!(
            error_string.contains("/test/path"),
            "Error message should contain the path"
        );
    }

    #[test]
    fn test_pattern_matching_error() {
        let regex_error = RegexError::Syntax("Invalid regex syntax".to_string());
        let error = pattern_matching_error(regex_error, "test-pattern");

        // Check that the error contains the expected information
        let error_string = format!("{error}");
        assert!(
            error_string.contains("test-pattern"),
            "Error message should contain the pattern"
        );
    }

    #[test]
    fn test_glob_pattern_error() {
        // Create a pattern that will cause an error
        let result = glob::Pattern::new("[");
        let pattern_error = result.err().unwrap();
        let error = glob_pattern_error(pattern_error, "test-glob-pattern");

        // Check that the error contains the expected information
        let error_string = format!("{error}");
        assert!(
            error_string.contains("test-glob-pattern"),
            "Error message should contain the pattern"
        );
    }

    #[test]
    fn test_path_operation_error() {
        let path = PathBuf::from("/test/path");
        let error = path_operation_error(path.clone(), "create");

        // Check that the error contains the expected information
        let error_string = format!("{error}");
        assert!(
            error_string.contains("create"),
            "Error message should contain the operation"
        );
        assert!(
            error_string.contains("/test/path"),
            "Error message should contain the path"
        );
    }

    #[test]
    fn test_config_parsing_error() {
        let io_error = io::Error::new(io::ErrorKind::InvalidData, "Invalid YAML");
        let error = config_parsing_error(io_error, "Missing required field");

        // Check that the error contains the expected information
        let error_string = format!("{error}");
        assert!(
            error_string.contains("Missing required field"),
            "Error message should contain the detail"
        );
    }

    #[test]
    fn test_pattern_extraction_error() {
        let error = pattern_extraction_error("<test>", "Missing closing bracket");

        // Check that the error contains the expected information
        let error_string = format!("{error}");
        assert!(
            error_string.contains("<test>"),
            "Error message should contain the pattern"
        );
        assert!(
            error_string.contains("Missing closing bracket"),
            "Error message should contain the detail"
        );
    }

    #[test]
    fn test_no_match_error() {
        let error = no_match_error("pattern", "value");

        // Check that the error contains the expected information
        let error_string = format!("{error}");
        assert!(
            error_string.contains("pattern"),
            "Error message should contain the pattern"
        );
        assert!(
            error_string.contains("value"),
            "Error message should contain the value"
        );
    }

    #[test]
    fn test_invalid_filename_error() {
        let path = PathBuf::from("/test/invalid:file");
        let error = invalid_filename_error(path.clone());

        // Check that the error contains the expected information
        let error_string = format!("{error}");
        assert!(
            error_string.contains("/test/invalid:file"),
            "Error message should contain the path"
        );
    }

    #[test]
    fn test_directory_not_found_error() {
        let path = PathBuf::from("/test/nonexistent");
        let error = directory_not_found_error(path.clone());

        // Check that the error contains the expected information
        let error_string = format!("{error}");
        assert!(
            error_string.contains("/test/nonexistent"),
            "Error message should contain the path"
        );
    }

    #[test]
    fn test_generic_error() {
        let error = generic_error("Something went wrong");

        // Check that the error contains the expected information
        let error_string = format!("{error}");
        assert!(
            error_string.contains("Something went wrong"),
            "Error message should contain the message"
        );
    }

    #[test]
    fn test_error_conversion() {
        // Test conversion from io::Error to Error
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error: Error = io_error.into();

        // Check that the error is converted correctly
        let error_string = format!("{error}");
        assert!(
            error_string.contains("Failed to perform operation on file"),
            "Error message should contain the underlying error"
        );

        // Test conversion from RegexError to Error
        let regex_error = RegexError::Syntax("Invalid regex syntax".to_string());
        let error: Error = regex_error.into();

        // Check that the error is converted correctly
        let error_string = format!("{error}");
        assert!(
            error_string.contains("Invalid pattern"),
            "Error message should contain the underlying error"
        );

        // Test conversion from PatternError to Error
        // Create a pattern that will cause an error
        let result = glob::Pattern::new("[");
        let pattern_error = result.err().unwrap();
        let error: Error = pattern_error.into();

        // Check that the error is converted correctly
        let error_string = format!("{error}");
        assert!(
            error_string.contains("Invalid glob pattern"),
            "Error message should contain the underlying error"
        );
    }
}
