//! File matching functionality
//!
//! This module contains functions for matching files against rules.

use anyhow::Result;
use log::{debug, info};
use regex::Regex;

use crate::config::Rule;
use crate::errors::pattern_matching_error;
use crate::logging::format_message;

use super::scanner::FileInfo;

/// Result of matching a file against a rule
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The file information
    pub file_info: FileInfo,
    /// The rule that matched the file
    pub rule: Rule,
    /// The root directory index from the rule
    pub root_index: usize,
}

/// Matches a file against a list of rules
///
/// This function checks if any of the rules match the given file.
///
/// # Arguments
/// * `file_info` - Information about the file to match
/// * `rules` - The list of rules to match against
///
/// # Returns
/// * `Result<Vec<MatchResult>>` - A list of match results or an error
///
/// # Errors
/// Returns an error if pattern matching fails
pub fn match_file_against_rules(file_info: &FileInfo, rules: &[Rule]) -> Result<Vec<MatchResult>> {
    let mut matches = Vec::new();

    for rule in rules {
        if let Ok(true) = does_rule_match(file_info, rule) {
            // Log the file found and rule being applied
            let message = format!(
                "{} found! Applying setup for {}.",
                file_info.filename, rule.title
            );
            // Create a simple colored message without direct use of the colored crate
            let colored_message = format!(
                "{} found! Applying setup for {}.",
                file_info.filename, rule.title
            );
            info!("{}", format_message(&message, &colored_message));

            matches.push(MatchResult {
                file_info: file_info.clone(),
                rule: rule.clone(),
                root_index: rule.root,
            });
        }
    }

    debug!(
        "Found {} matching rules for file: {}",
        matches.len(),
        file_info.filename
    );

    Ok(matches)
}

/// Checks if a rule matches a file
///
/// This function checks if the given rule matches the given file.
///
/// # Arguments
/// * `file_info` - Information about the file to match
/// * `rule` - The rule to match against
///
/// # Returns
/// * `Result<bool>` - True if the rule matches, false otherwise, or an error
///
/// # Errors
/// Returns an error if pattern matching fails
fn does_rule_match(file_info: &FileInfo, rule: &Rule) -> Result<bool> {
    let pattern =
        Regex::new(&rule.old_pattern).map_err(|e| pattern_matching_error(e, &rule.old_pattern))?;
    Ok(pattern.is_match(&file_info.filename))
}
