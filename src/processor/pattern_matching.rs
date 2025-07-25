//! Pattern matching functionality
//!
//! This module contains methods for pattern matching and extraction,
//! including resolving group substrings and parsing files.

use super::convert_file_format;
use super::core::Processor;
use crate::errors::{generic_error, no_match_error, pattern_matching_error, Result};
use crate::processor::format_conversion::{SUPPORTED_IMAGE_FORMATS, SUPPORTED_TEXT_ENCODINGS};
use crate::rules::Rule;
use crate::utils::{process_date, process_pattern};
use regex::{Match, Regex};
use std::path::{Path, PathBuf};

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
        run_execution: bool,
    ) -> Result<(PathBuf, PathBuf)> {
        let mut processed_value: String = self.parse_file(new_name)?;
        let root = match root {
            None => self.target(),
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

        let target_path = root.join(PathBuf::from(processed_value));

        // Apply format conversion if specified
        if let Some(config_processor) = &rule.processors
            && let Some(format_conversion) = &config_processor.format_conversion
        {
            // Create a temporary path for the converted file
            let source_path = self.source().to_path_buf();

            if (SUPPORTED_IMAGE_FORMATS.contains(&format_conversion.source_format.as_str())
                && SUPPORTED_IMAGE_FORMATS
                    .contains(&source_path.extension().unwrap().to_str().unwrap()))
                || SUPPORTED_TEXT_ENCODINGS.contains(&format_conversion.source_format.as_str())
            {
                // Apply the format conversion
                let converted_path = convert_file_format(
                    &source_path,
                    &target_path,
                    format_conversion,
                    run_execution,
                )?;
                return Ok(converted_path);
            }
        }

        Ok((self.source().to_path_buf(), target_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processor::ProcessorBuilder;
    use crate::utils::{clean_pattern, extract_pattern};

    #[test]
    fn test_clean_pattern() {
        // Test with angle brackets
        let result = clean_pattern("<pattern>");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "pattern");

        // Test with multiple angle brackets
        let result = clean_pattern("<pat<ter>n>");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "pattern");

        // Test with no angle brackets
        let result = clean_pattern("pattern");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "pattern");

        // Test with empty string
        let result = clean_pattern("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_extract_pattern() {
        // Test with angle brackets
        let result = extract_pattern("<pattern>");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "pattern");

        // Test with no angle brackets
        let result = extract_pattern("pattern");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "pattern");

        // Test with empty angle brackets
        let result = extract_pattern("<>");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");

        // Test with nested angle brackets (should only extract the innermost content)
        let result = extract_pattern("<outer<inner>>");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "inner");
    }

    #[test]
    fn test_process_pattern() {
        // Test with valid pattern and replacement
        let mut destination = "test_filename.txt".to_string();
        let pattern = "test";
        let replacement = Some("replaced".to_string());

        let result = process_pattern(&mut destination, pattern, &replacement);
        assert!(result.is_ok());
        assert_eq!(destination, "replaced_filename.txt");

        // Test with regex pattern
        let mut destination = "test123_filename.txt".to_string();
        let pattern = r"test\d+";
        let replacement = Some("replaced".to_string());

        let result = process_pattern(&mut destination, pattern, &replacement);
        assert!(result.is_ok());
        assert_eq!(destination, "replaced_filename.txt");

        // Test with no replacement (should not change the string)
        let mut destination = "test_filename.txt".to_string();
        let pattern = "test";
        let replacement = None;

        let result = process_pattern(&mut destination, pattern, &replacement);
        assert!(result.is_ok());
        assert_eq!(destination, "test_filename.txt");

        // Test with invalid regex pattern (should return an error)
        let mut destination = "test_filename.txt".to_string();
        let pattern = "["; // Invalid regex pattern
        let replacement = Some("replaced".to_string());

        let result = process_pattern(&mut destination, pattern, &replacement);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid pattern"));
    }

    #[test]
    fn test_process_date() {
        // Test with valid timestamp and splitter
        let mut destination = "1626912000_filename.txt".to_string();
        let fmt = "%Y-%m-%d";
        let splitter = "_";
        let merger = Some(" ".to_string());

        let result = process_date(&mut destination, fmt, splitter, &merger);
        assert!(result.is_ok());
        assert_eq!(destination, "2021-07-22 filename.txt");

        // Test with different format
        let mut destination = "1626912000_filename.txt".to_string();
        let fmt = "%d/%m/%Y";
        let splitter = "_";
        let merger = Some("-".to_string());

        let result = process_date(&mut destination, fmt, splitter, &merger);
        assert!(result.is_ok());
        assert_eq!(destination, "22/07/2021-filename.txt");

        // Test with invalid timestamp (should return an error)
        let mut destination = "invalid_filename.txt".to_string();
        let fmt = "%Y-%m-%d";
        let splitter = "_";
        let merger = Some(" ".to_string());

        let result = process_date(&mut destination, fmt, splitter, &merger);
        assert!(result.is_err());

        // Test with missing splitter (should return an error)
        let mut destination = "1626912000filename.txt".to_string();
        let fmt = "%Y-%m-%d";
        let splitter = "_";
        let merger = Some(" ".to_string());

        let result = process_date(&mut destination, fmt, splitter, &merger);
        assert!(result.is_err());
    }

    #[test]
    fn test_processor_builder() {
        // Create a test processor with a filename
        let processor = ProcessorBuilder::new(Path::new("test_123_filename.txt"))
            .target(PathBuf::from("target_dir"))
            .build();

        // Verify that the source path is set correctly
        assert_eq!(processor.source(), &PathBuf::from("test_123_filename.txt"));

        // Verify that the target path is set correctly
        assert_eq!(processor.target(), &PathBuf::from("target_dir"));
    }

    #[test]
    fn test_format_conversion_integration() {
        use crate::rules::{ConfigProcessor, FormatConversion, Rule};
        use std::fs;
        use tempfile::tempdir;

        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test.txt");
        let target_dir = temp_dir.path().join("target");
        fs::create_dir_all(&target_dir).unwrap();

        // Create a test text file in UTF-8
        let text = "Hello, world! 你好，世界！";
        fs::write(&source_path, text).unwrap();

        // Create a processor with the source file
        let processor = ProcessorBuilder::new(&source_path)
            .target(target_dir.clone())
            .build();

        // Create a rule with format conversion
        let rule = Rule {
            title: "Test Format Conversion".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: None,
            function: None,
            processors: Some(ConfigProcessor {
                splitter: None,
                merger: None,
                pattern: None,
                date_format: None,
                replacement: None,
                format_conversion: Some(FormatConversion {
                    source_format: "utf-8".to_string(),
                    target_format: "utf-16".to_string(),
                    resize: None,
                }),
            }),
            root: 0,
            copy: false,
            old_pattern: "pattern".to_string(),
            new_pattern: "pattern".to_string(),
        };

        // Call make_destination with the rule
        let result = processor.make_destination("output.txt", Some(&target_dir), &rule, true);
        assert!(result.is_ok());

        let (_, target_path) = result.unwrap();
        assert!(target_path.exists());

        // Read the converted file
        let mut file = fs::File::open(&target_path).unwrap();
        let mut buffer = Vec::new();
        use std::io::Read;
        file.read_to_end(&mut buffer).unwrap();

        // Decode the content using UTF-16LE
        let (decoded, _, _) = encoding_rs::UTF_16LE.decode(&buffer);
        assert_eq!(decoded, text);
    }
}
