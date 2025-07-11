//! Pattern matching functionality
//!
//! This module contains methods for pattern matching and extraction,
//! including resolving group substrings and parsing files.

use crate::errors::{generic_error, no_match_error, pattern_matching_error, Result};
use crate::rules::Rule;
use crate::utils::{process_date, process_pattern};
use regex::{Match, Regex};
use std::path::{Path, PathBuf};

use super::core::Processor;

impl Processor {
    /// Resolves a substring from the source filename based on a range
    ///
    /// This method extracts a substring from the source filename using the provided
    /// range information. The range is specified as [start, length].
    ///
    /// # Arguments
    /// * `range` - A vector containing the start index and length of the substring to extract
    ///
    /// # Returns
    /// * `Result<String>` - The extracted substring, or an error if the range is invalid
    ///
    /// # Errors
    /// * Returns an error if the range format is invalid (less than 2 elements)
    /// * Returns an error if the range is out of bounds for the filename
    pub(crate) fn resolve_group_substring(&self, range: Vec<usize>) -> Result<String> {
        if range.len() < 2 {
            return Err(generic_error(
                "Invalid range format: expected [start, length]",
            ));
        }

        let range_start = range[0];
        let range_end = range[0] + range[1];

        let filename = self.source_filename()?;
        if range_start >= filename.len() || range_end > filename.len() {
            return Err(generic_error(&format!(
                "Range [{}, {}] is out of bounds for filename of length {}",
                range_start,
                range_end,
                filename.len()
            )));
        }

        Ok(filename[range_start..range_end].to_string())
    }

    /// Extracts group values from a regex match, parsing them as usize
    ///
    /// This method takes a regex match that contains a pattern like "1:3" and
    /// splits it by the colon character, parsing each part as an unsigned integer.
    ///
    /// # Arguments
    /// * `found_group` - An optional regex match containing the group values
    ///
    /// # Returns
    /// * `Vec<usize>` - A vector of parsed unsigned integers, or an empty vector if no match
    pub(crate) fn extract_group_values(&self, found_group: Option<Match>) -> Vec<usize> {
        match found_group {
            Some(group) => group
                .as_str()
                .split(':')
                .filter_map(|s| s.parse::<usize>().ok())
                .collect(),
            None => Vec::new(),
        }
    }

    /// Parses the source filename using a regex pattern
    ///
    /// This method applies a regex pattern to the source filename and returns
    /// either the matched part or the entire filename if no specific group is matched.
    ///
    /// # Arguments
    /// * `pattern` - The regex pattern to match against the source filename
    ///
    /// # Returns
    /// * `Result<String>` - The matched part of the filename or the entire filename, or an error
    ///
    /// # Errors
    /// * Returns an error if the regex pattern is invalid
    /// * Returns an error if the pattern doesn't match the source filename
    pub(crate) fn parse_file(&self, pattern: &str) -> Result<String> {
        let source_filename = self.source_filename()?.to_string();

        let regex = Regex::new(pattern).map_err(|e| pattern_matching_error(e, pattern))?;

        let captures = regex
            .captures(&source_filename)
            .ok_or_else(|| no_match_error(pattern, &source_filename))?;

        let group = captures.get(0);

        Ok(if let Some(g) = group {
            g.as_str().to_string()
        } else {
            source_filename
        })
    }

    /// Creates a destination path based on the rule and pattern
    ///
    /// This method generates the final destination path for a file by applying
    /// pattern matching and processing rules. It can transform the filename
    /// based on date formatting and pattern replacement.
    ///
    /// # Arguments
    /// * `new_name` - The pattern to match against the source filename
    /// * `root` - Optional root directory to use instead of the target directory
    /// * `rule` - The rule containing processing instructions
    ///
    /// # Returns
    /// * `Result<PathBuf>` - The final destination path, or an error
    ///
    /// # Errors
    /// * Returns an error if pattern matching fails
    /// * Returns an error if date or pattern processing fails
    pub(crate) fn make_destination(
        &self,
        new_name: &str,
        root: Option<&Path>,
        rule: &Rule,
    ) -> Result<PathBuf> {
        let mut processed_value: String = self.parse_file(new_name)?;
        let root = match root {
            None => &self.target,
            Some(r) => r,
        };

        if let Some(config_processor) = &rule.processors {
            // Process date if both date_format and splitter are provided
            if let (Some(date_format), Some(splitter)) =
                (&config_processor.date_format, &config_processor.splitter)
            {
                process_date(
                    &mut processed_value,
                    date_format,
                    splitter,
                    &config_processor.merger,
                )?;
            }

            // Process pattern if provided
            if let Some(pattern) = &config_processor.pattern {
                process_pattern(&mut processed_value, pattern, &config_processor.replacement)?;
            }
        }

        Ok(root.join(PathBuf::from(processed_value)))
    }
}