//! Configuration serialization functionality
//!
//! This module contains functions for serializing configuration structures to YAML.

use std::path::{Path, PathBuf};

use serde::{Serialize, Serializer};

use crate::config::model::RulesList;

fn process_pathbuf(path: &Path) -> Vec<String> {
    path.iter()
        .map(|component| component.to_string_lossy().to_string())
        .filter(|component| component != "/" && component != "\\")
        .map(|component| {
            if component.ends_with(':') {
                format!("{component}/")
            } else {
                component
            }
        })
        .collect()
}

/// Serializes a PathBuf vector to an array of arrays of strings
///
/// This function is used to serialize the root directories field in a Config struct.
pub fn serialize_pathbuf_vec_to_arrays<S>(
    paths: &[PathBuf],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Convert each PathBuf to a vector of string components
    let path_components: Vec<Vec<String>> =
        paths.iter().map(|path| process_pathbuf(path)).collect();

    // Serialize the vector of vectors
    path_components.serialize(serializer)
}

/// Serializes a PathBuf to an array of strings
///
/// This function is used to serialize the download directory field in a Config struct.
pub fn serialize_pathbuf_to_array<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Convert PathBuf to a vector of string components
    let components: Vec<String> = process_pathbuf(path);

    // Serialize the vector
    components.serialize(serializer)
}

/// Serializes an optional PathBuf to an array of strings or null
///
/// This function is used to serialize the directory field in a Rule struct.
pub fn serialize_optional_pathbuf_to_array<S>(
    path: &Option<PathBuf>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match path {
        Some(path) => {
            // Convert PathBuf to a vector of string components
            let components: Vec<String> = process_pathbuf(path);

            // Serialize the vector
            components.serialize(serializer)
        }
        None => serializer.serialize_none(),
    }
}

/// Serializes a RulesList to a YAML value
///
/// This function is used to serialize the rules field in a Config struct.
pub fn serialize_rules<S>(rules: &RulesList, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // For simplicity, we'll serialize as a single list of rules
    rules.serialize(serializer)
}
