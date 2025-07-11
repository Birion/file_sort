use std::path::{Path, PathBuf};

use glob::glob;
use serde::Deserialize;

use crate::constants::WILDCARD;
use crate::errors::{directory_not_found_error, file_operation_error, glob_pattern_error, invalid_filename_error, Error, Result};

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
pub enum TransformativeFunction {
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

impl TransformativeFunction {
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
            .ok_or_else(|| invalid_filename_error(path.clone()))?;

        let pattern_results = glob(path_str).map_err(|e| glob_pattern_error(e, path_str))?;

        let results: Vec<PathBuf> = pattern_results
            .map(|res| {
                res.map_err(|e| file_operation_error(e.into_error(), path.clone(), "access"))
            })
            .collect::<std::result::Result<Vec<PathBuf>, Error>>()?;

        if results.is_empty() {
            return Err(directory_not_found_error(path));
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
            TransformativeFunction::Last { args } => args,
            TransformativeFunction::First { args } => args,
        };
        match args {
            Some(arg) => {
                for x in arg {
                    path.push(x)
                }
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
            TransformativeFunction::Last { .. } => Ok(results[results.len() - 1].clone()),
            TransformativeFunction::First { .. } => Ok(results[0].clone()),
        }
    }
}