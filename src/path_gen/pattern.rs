//! Pattern matching and replacement
//!
//! This module contains functions for pattern matching and replacement to generate destination paths.

use std::path::PathBuf;

use anyhow::{Result, anyhow};
use log::{debug, info};
use regex::Regex;

use crate::config::{ConfigProcessor, Rule};
use crate::errors::{no_match_error, pattern_matching_error};
use crate::utils::{process_date, process_pattern};

use super::transformer::TransformResult;

/// Result of generating a destination path
#[derive(Debug, Clone)]
pub struct PathResult {
    /// The source path
    pub source_path: PathBuf,
    /// The target path
    pub target_path: PathBuf,
    /// The rule that was applied
    pub rule: Rule,
}

/// Generates a destination path based on a rule and pattern
///
/// This function generates the final destination path for a file by applying
/// pattern matching and processing rules. It can transform the filename
/// based on date formatting and pattern replacement.
///
/// # Arguments
/// * `transform_result` - The result of applying a transformative function
/// * `run_execution` - Whether to actually perform the file operations (false) or just simulate them (true)
///
/// # Returns
/// * `Result<PathResult>` - The final destination path, or an error
///
/// # Errors
/// * Returns an error if pattern matching fails
/// * Returns an error if date or pattern processing fails
pub fn generate_destination_path(
    transform_result: &TransformResult,
    _run_execution: bool, // Prefix with underscore to indicate intentionally unused
) -> Result<PathResult> {
    let source_path = &transform_result.source_path;
    let target_dir = &transform_result.target_dir;
    let rule = &transform_result.rule;

    // Get the source filename
    let source_filename = source_path
        .file_name()
        .ok_or_else(|| {
            anyhow!(
                "Failed to get filename from path: {}",
                source_path.display()
            )
        })?
        .to_str()
        .ok_or_else(|| anyhow!("Invalid filename: {}", source_path.display()))?;

    // Parse the file using the rule's pattern
    let mut processed_value = parse_file(source_filename, &rule.old_pattern)?;

    // Apply processors if defined
    if let Some(config_processor) = &rule.processors {
        apply_processors(&mut processed_value, config_processor)?;
    }

    // Create the target path
    let target_path = target_dir.join(processed_value);

    // Log the new filename if changed
    if source_filename != target_path.file_name().unwrap().to_str().unwrap() {
        let target_filename = target_path.file_name().unwrap().to_str().unwrap();
        let message = format!("New filename: {}", target_filename);
        let colored_message = format!(
            "New filename: {}",
            colored::Colorize::bold(colored::Colorize::red(target_filename))
        );
        info!(
            "{}",
            crate::logging::format_message(&message, &colored_message)
        );
    }

    debug!("Generated target path: {}", target_path.display());

    Ok(PathResult {
        source_path: source_path.to_path_buf(),
        target_path,
        rule: rule.clone(),
    })
}

/// Parses a filename using a regex pattern
///
/// This method applies a regex pattern to the filename and returns
/// either the matched part or the entire filename if no specific group is matched.
///
/// # Arguments
/// * `filename` - The filename to parse
/// * `pattern` - The regex pattern to match against the filename
///
/// # Returns
/// * `Result<String>` - The matched part of the filename or the entire filename, or an error
///
/// # Errors
/// * Returns an error if the regex pattern is invalid
/// * Returns an error if the pattern doesn't match the filename
fn parse_file(filename: &str, pattern: &str) -> Result<String> {
    let regex = Regex::new(pattern).map_err(|e| pattern_matching_error(e, pattern))?;

    let captures = regex
        .captures(filename)
        .ok_or_else(|| no_match_error(pattern, filename))?;

    let group = captures.get(0);

    Ok(if let Some(g) = group {
        g.as_str().to_string()
    } else {
        filename.to_string()
    })
}

/// Applies processors to a filename
///
/// This function applies date formatting and pattern replacement to a filename.
///
/// # Arguments
/// * `processed_value` - The filename to process
/// * `config_processor` - The processor configuration
///
/// # Returns
/// * `Result<()>` - Success or an error
///
/// # Errors
/// * Returns an error if date or pattern processing fails
fn apply_processors(
    processed_value: &mut String,
    config_processor: &ConfigProcessor,
) -> Result<()> {
    // Process date if both date_format and splitter are provided
    if let (Some(date_format), Some(splitter)) =
        (&config_processor.date_format, &config_processor.splitter)
    {
        process_date(
            processed_value,
            date_format,
            splitter,
            &config_processor.merger,
        )?;
    }

    // Process pattern if provided
    if let Some(pattern) = &config_processor.pattern {
        process_pattern(processed_value, pattern, &config_processor.replacement)?;
    }

    Ok(())
}
