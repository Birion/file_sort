//! Transformative function application
//!
//! This module contains functions for applying transformative functions to generate destination paths.

use std::path::{Path, PathBuf};

use anyhow::Result;
use glob::glob;
use log::debug;
use serde::Deserialize;

use crate::config::Rule;
use crate::constants::WILDCARD;
use crate::utils::full_path;

/// A list of arguments for transformative functions.
///
/// Used to provide parameters to transformative functions like First and Last.
pub type ArgumentList = Vec<String>;

/// Function that transforms paths based on specific rules
///
/// This enum represents transformative functions that can be applied to directory paths.
/// These functions are used to select specific directories from a set of matching directories
/// based on criteria like position (first, last).
///
/// Transformative functions are particularly useful when dealing with versioned directories
/// or directories that follow a specific naming pattern, allowing the selection of the most
/// recent or oldest directory automatically.
///
/// # Examples
///
/// In a configuration file, a transformative function might be used like this:
///
/// ```yaml
/// function:
///   name: last
///   args:
///     - "comics"
///     - "batman"
/// ```
///
/// This would select the last (most recent) directory that matches the pattern "comics/batman/*".
///
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "name")]
pub enum FolderFunction {
    /// Selects the last directory that matches the pattern
    ///
    /// This variant is used when you want to select the last directory
    /// in a sequence of directories that match a pattern. This is often
    /// used to select the most recent version of a directory.
    ///
    /// # Fields
    /// * `args` - Optional list of path components to use when constructing the path pattern
    Last { args: Option<ArgumentList> },

    /// Selects the first directory that matches the pattern
    ///
    /// This variant is used when you want to select the first directory
    /// in a sequence of directories that match a pattern. This is often
    /// used to select the oldest version of a directory.
    ///
    /// # Fields
    /// * `args` - Optional list of path components to use when constructing the path pattern
    First { args: Option<ArgumentList> },
}

impl FolderFunction {
    /// Gets the directory based on the transformative function
    ///
    /// # Arguments
    /// * `root` - The root directory to start from
    ///
    /// # Returns
    /// * `Result<PathBuf>` - The selected directory path or an error
    pub fn get_dir(&self, root: &Path) -> Result<PathBuf> {
        let path = self.construct_path(root);
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", path.display()))?;

        let pattern_results = glob(path_str)
            .map_err(|e| anyhow::anyhow!("Invalid glob pattern '{}': {}", path_str, e))?;

        let results: Vec<PathBuf> = pattern_results
            .map(|res| {
                res.map_err(|e| anyhow::anyhow!("Failed to access {}: {}", path.display(), e))
            })
            .collect::<Result<Vec<PathBuf>>>()?;

        if results.is_empty() {
            return Err(anyhow::anyhow!("Directory not found: {}", path.display()));
        }

        self.get_result_based_on_transformation(results)
    }

    /// Constructs a path based on the function's arguments
    ///
    /// # Arguments
    /// * `root` - The root directory to start from
    ///
    /// # Returns
    /// * `PathBuf` - The constructed path
    fn construct_path(&self, root: &Path) -> PathBuf {
        let mut path: PathBuf = root.into();
        let args = match self {
            FolderFunction::Last { args } => args,
            FolderFunction::First { args } => args,
        };
        match args {
            Some(arg) => {
                for x in arg {
                    path.push(x)
                }
                path.push(WILDCARD);
            }
            None => path.push(WILDCARD),
        }

        path
    }

    /// Selects a result from a list of paths based on the transformation type
    ///
    /// # Arguments
    /// * `results` - A list of paths to select from
    ///
    /// # Returns
    /// * `Result<PathBuf>` - The selected path or an error
    fn get_result_based_on_transformation(&self, results: Vec<PathBuf>) -> Result<PathBuf> {
        match self {
            FolderFunction::Last { .. } => Ok(results[results.len() - 1].clone()),
            FolderFunction::First { .. } => Ok(results[0].clone()),
        }
    }
}

/// Result of applying a transformative function
#[derive(Debug, Clone)]
pub struct TransformResult {
    /// The source path
    pub source_path: PathBuf,
    /// The target directory
    pub target_dir: PathBuf,
    /// The rule that was applied
    pub rule: Rule,
}

/// Applies a transformative function to generate a destination path
///
/// This function applies the transformative function specified in the rule to generate
/// a destination path for the file.
///
/// # Arguments
/// * `source_path` - The path to the source file
/// * `root_paths` - The list of root directories
/// * `rule` - The rule containing the transformative function
///
/// # Returns
/// * `Result<TransformResult>` - The result of applying the transformative function or an error
///
/// # Errors
/// Returns an error if the transformative function fails
pub fn apply_transformative_function(
    source_path: &Path,
    root_paths: &[PathBuf],
    rule: &Rule,
) -> Result<TransformResult> {
    let root_path = &root_paths[rule.root];

    // Determine the target directory
    let target_dir = if let Some(function) = &rule.function {
        // Apply the transformative function
        debug!("Applying transformative function for rule: {}", rule.title);
        function.get_dir(root_path)?
    } else if let Some(dir) = &rule.directory {
        // Use the specified directory
        full_path(root_path, dir)
    } else {
        // Use the rule title as the directory
        root_path.join(&rule.title)
    };

    debug!("Target directory: {}", target_dir.display());

    Ok(TransformResult {
        source_path: source_path.to_path_buf(),
        target_dir,
        rule: rule.clone(),
    })
}
