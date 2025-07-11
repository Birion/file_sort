//! Path handling functionality
//!
//! This module contains methods for handling file paths, including
//! extracting filenames and parsing directory paths.

use crate::errors::{invalid_filename_error, path_operation_error, pattern_extraction_error, pattern_matching_error, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};

use super::core::Processor;

impl Processor {
    /// Checks if the target filename is different from the source filename
    ///
    /// This is used to determine if a file needs to be renamed during processing.
    ///
    /// # Returns
    /// * `Result<bool>` - True if the filenames are different, false otherwise, or an error
    pub(crate) fn is_changed(&self) -> Result<bool> {
        let target_filename = self.target_filename()?;
        let source_filename = self.source_filename()?;
        Ok(target_filename != source_filename)
    }

    /// Gets the source filename as a string
    ///
    /// Extracts the filename component from the source path and converts it to a string.
    ///
    /// # Returns
    /// * `Result<&str>` - The filename as a string, or an error if the filename cannot be extracted or converted
    pub(crate) fn source_filename(&self) -> Result<&str> {
        self.source
            .file_name()
            .ok_or_else(|| path_operation_error(self.source.clone(), "get filename"))
            .and_then(|os_str| {
                os_str
                    .to_str()
                    .ok_or_else(|| invalid_filename_error(self.source.clone()))
            })
    }

    /// Gets the target filename as a string
    ///
    /// Extracts the filename component from the target path and converts it to a string.
    ///
    /// # Returns
    /// * `Result<&str>` - The filename as a string, or an error if the filename cannot be extracted or converted
    pub(crate) fn target_filename(&self) -> Result<&str> {
        self.target
            .file_name()
            .ok_or_else(|| path_operation_error(self.target.clone(), "get filename"))
            .and_then(|os_str| {
                os_str
                    .to_str()
                    .ok_or_else(|| invalid_filename_error(self.target.clone()))
            })
    }

    /// Parses a directory path, resolving any pattern groups
    ///
    /// This method processes a directory path that may contain pattern groups
    /// enclosed in angle brackets (e.g. "<1:3>"). It extracts the pattern,
    /// resolves it using the source filename, and replaces it in the path.
    ///
    /// # Arguments
    /// * `directory` - The directory path to parse
    ///
    /// # Returns
    /// * `Result<PathBuf>` - The parsed directory path with resolved patterns, or an error
    ///
    /// # Errors
    /// * Returns an error if the directory path cannot be converted to a string
    /// * Returns an error if pattern extraction or matching fails
    pub(crate) fn parse_dir(&self, directory: &Path) -> Result<PathBuf> {
        static GROUP_PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r".*<(.*)>.*").expect("Failed to compile regex pattern for GROUP_PATTERN")
        });

        let directory_string = directory
            .to_str()
            .ok_or_else(|| invalid_filename_error(directory.to_path_buf()))?;

        if !GROUP_PATTERN.is_match(directory_string) {
            return Ok(directory.to_path_buf());
        }

        let group_match = GROUP_PATTERN
            .find(directory_string)
            .ok_or_else(|| pattern_extraction_error(directory_string, "Failed to find a match"))?
            .as_str();

        let found_group = GROUP_PATTERN
            .captures(group_match)
            .ok_or_else(|| pattern_extraction_error(group_match, "Failed to capture groups"))?
            .get(1)
            .ok_or_else(|| pattern_extraction_error(group_match, "No capture group found"))?;

        let group_values = self.extract_group_values(Some(found_group));
        let replace_part = self.resolve_group_substring(group_values)?;

        let pattern_str = format!("<{}>", found_group.as_str());
        let new_pattern =
            Regex::new(&pattern_str).map_err(|e| pattern_matching_error(e, &pattern_str))?;

        let dir = new_pattern
            .replace(directory_string, &replace_part)
            .to_string();
        Ok(PathBuf::from(dir))
    }
}
