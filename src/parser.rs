use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

use crate::rules::{Rules, RulesList};
use utils::*;

mod utils;

/// Deserialises an array of strings into a PathBuf
///
/// This function is used as a custom deserializer for Serde.
///
/// # Arguments
/// * `deserializer` - The deserializer to use
///
/// # Returns
/// * `Result<PathBuf, D::Error>` - The deserialized PathBuf or an error
pub fn deserialize_from_array_to_pathbuf<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path_strings: Vec<String> = Deserialize::deserialize(deserializer)?;
    Ok(path_strings.iter().map(process_path).collect())
}

/// Deserialises an array of strings into an optional PathBuf
///
/// This function is used as a custom deserializer for Serde.
///
/// # Arguments
/// * `deserializer` - The deserializer to use
///
/// # Returns
/// * `Result<Option<PathBuf>, D::Error>` - The deserialised optional PathBuf or an error
pub fn deserialize_from_array_to_optional_pathbuf<'de, D>(
    deserializer: D,
) -> Result<Option<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let result: Option<Vec<String>> = Deserialize::deserialize(deserializer)?;

    match result {
        None => Ok(None),
        Some(p) => Ok(Some(p.iter().map(process_path).collect())),
    }
}

/// Deserialises arrays of strings into a vector of PathBufs
///
/// This function is used as a custom deserializer for Serde.
///
/// # Arguments
/// * `deserializer` - The deserializer to use
///
/// # Returns
/// * `Result<Vec<PathBuf>, D::Error>` - The deserialised vector of PathBufs or an error
pub fn deserialize_from_arrays_to_pathbuf_vec<'de, D>(
    deserializer: D,
) -> Result<Vec<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let paths: Vec<Vec<String>> = Deserialize::deserialize(deserializer)?;
    Ok(paths.into_iter().map(process_strings_to_paths).collect())
}

/// Parses rules from the configuration
///
/// This function is used as a custom deserializer for Serde.
/// It handles both single rule lists and multiple rule lists organised by root directories.
///
/// # Arguments
/// * `deserializer` - The deserializer to use
///
/// # Returns
/// * `Result<RulesList, D::Error>` - The parsed list of rules or an error
pub fn parse_rules<'de, D>(deserializer: D) -> Result<RulesList, D::Error>
where
    D: Deserializer<'de>,
{
    let parsed_rules: Rules = Deserialize::deserialize(deserializer)?;
    let mut result_rules = vec![];
    match parsed_rules {
        Rules::SingleRule(mut rules) => {
            process_rules(&mut rules, &mut result_rules);
        }
        Rules::RootRules(roots) => {
            process_and_append_rules(roots, &mut result_rules);
        }
    }
    result_rules.dedup();
    Ok(result_rules)
}

pub fn default_merger() -> Option<String> {
    Some(String::from("-"))
}
