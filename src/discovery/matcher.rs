//! File matching functionality
//!
//! This module contains functions for matching files against rules.

use anyhow::Result;
use log::{debug, info};
use regex::Regex;

use crate::config::Rule;
use crate::discovery::content_analyser::evaluate_condition;
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
/// It supports both pattern-based matching and content-based matching.
///
/// # Arguments
/// * `file_info` - Information about the file to match
/// * `rule` - The rule to match against
///
/// # Returns
/// * `Result<bool>` - True if the rule matches, false otherwise, or an error
///
/// # Errors
/// Returns an error if pattern matching or content analysis fails
fn does_rule_match(file_info: &FileInfo, rule: &Rule) -> Result<bool> {
    // Check pattern-based matching first
    let pattern_match = if !rule.old_pattern.is_empty() {
        let pattern = Regex::new(&rule.old_pattern)
            .map_err(|e| pattern_matching_error(e, &rule.old_pattern))?;
        pattern.is_match(&file_info.filename)
    } else if let Some(patterns) = &rule.patterns {
        // Check if any of the patterns match
        patterns.iter().any(|p| {
            if let Ok(regex) = Regex::new(p) {
                regex.is_match(&file_info.filename)
            } else {
                false
            }
        })
    } else {
        // If no patterns are specified, consider it a match for content-based rules
        rule.content_conditions.is_some()
    };

    // If pattern doesn't match or there are no content conditions, return the pattern match result
    if !pattern_match || rule.content_conditions.is_none() {
        return Ok(pattern_match);
    }

    // If we have content conditions, check them
    if let Some(conditions) = &rule.content_conditions {
        // If there are no conditions, consider it a match
        if conditions.is_empty() {
            return Ok(true);
        }

        // Clone file_info to get a mutable version for content analysis
        let mut file_info_mut = file_info.clone();

        // Ensure content is analysed
        let content_analysis = file_info_mut.ensure_content_analysed()?;

        // Evaluate each condition
        let mut all_match = true;
        let mut any_match = false;

        for condition in conditions {
            match evaluate_condition(condition, content_analysis) {
                Ok(matches) => {
                    if matches {
                        any_match = true;
                    } else {
                        all_match = false;
                    }
                }
                Err(e) => {
                    debug!("Error evaluating condition: {}", e);
                    all_match = false;
                }
            }
        }

        // Return based on match_all_conditions flag
        Ok(if rule.match_all_conditions {
            all_match
        } else {
            any_match
        })
    } else {
        // If there are no content conditions, the pattern match is sufficient
        Ok(pattern_match)
    }
}
