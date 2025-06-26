use crate::errors::{
    file_operation_error, generic_error, invalid_filename_error, no_match_error,
    path_operation_error, pattern_extraction_error, pattern_matching_error, Result,
};
use crate::rules::Rule;
use crate::utils::{full_path, process_date, process_pattern};
use once_cell::sync::Lazy;
use regex::{Match, Regex};
use std::fs::{copy, create_dir_all, rename};
use std::path::{Path, PathBuf};

/// Handles file processing operations for sorting files
///
/// The Processor struct is responsible for managing the source and target paths
/// of files being processed and performing operations like copying, moving,
/// and pattern matching on these files.
#[derive(Debug, Clone)]
pub(crate) struct Processor {
    /// The source path of the file being processed
    source: PathBuf,
    /// The target path where the file will be moved or copied to
    pub(crate) target: PathBuf,
}

impl Processor {
    /// Creates a new Processor instance for the given file
    ///
    /// # Arguments
    /// * `file` - The path to the file to be processed
    ///
    /// # Returns
    /// * `Processor` - A new Processor instance with the source set to the given file
    pub(crate) fn new(file: &Path) -> Processor {
        Processor {
            source: file.to_path_buf(),
            target: PathBuf::new(),
        }
    }

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

    /// Performs the file action (copy or move)
    ///
    /// This is a convenience method that determines whether to rename or copy a file
    /// based on the provided boolean flag.
    ///
    /// # Arguments
    /// * `is_copy_operation` - If true, the file will be copied; if false, it will be moved
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the operation succeeds, or an error if it fails
    pub(crate) fn perform_file_action(&self, is_copy_operation: bool) -> Result<()> {
        let is_rename_operation = !is_copy_operation;
        self.perform_file_operation(is_copy_operation, is_rename_operation)
    }

    /// Performs the actual file operation (copy and/or rename)
    ///
    /// This method handles the low-level file system operations for copying and/or
    /// renaming files. It can be configured to perform either or both operations.
    ///
    /// # Arguments
    /// * `is_copy_operation` - If true, the file will be copied
    /// * `is_rename_operation` - If true, the file will be renamed (moved)
    ///
    /// # Returns
    /// * `Result<()>` - Ok if all operations succeed, or an error if any operation fails
    pub(crate) fn perform_file_operation(
        &self,
        is_copy_operation: bool,
        is_rename_operation: bool,
    ) -> Result<()> {
        if is_copy_operation {
            copy(&self.source, &self.target)
                .map_err(|e| file_operation_error(e, self.source.clone(), "copy"))?;
        }
        if is_rename_operation {
            rename(&self.source, &self.target)
                .map_err(|e| file_operation_error(e, self.source.clone(), "move"))?;
        }
        Ok(())
    }

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

    /// Creates and sets the target directory for the file
    ///
    /// This method creates the target directory structure and sets the target path
    /// for the processor. It also resolves any patterns in the directory path.
    ///
    /// # Arguments
    /// * `root` - The root directory where files will be moved or copied to
    /// * `folder` - The specific folder within the root directory
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the directory was created successfully, or an error
    ///
    /// # Errors
    /// * Returns an error if directory creation fails
    /// * Returns an error if pattern parsing fails
    pub(crate) fn create_and_set_target_directory(
        &mut self,
        root: &Path,
        folder: &Path,
    ) -> Result<()> {
        let folder_full_path = full_path(root, folder);

        self.target = self.parse_dir(&folder_full_path)?;

        create_dir_all(&self.target)
            .map_err(|e| file_operation_error(e, self.target.clone(), "create directory"))?;

        Ok(())
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
