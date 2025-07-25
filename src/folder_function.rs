use std::path::{Path, PathBuf};

use glob::glob;
use serde::{Deserialize, Serialize};

use crate::constants::WILDCARD;
use crate::errors::{
    directory_not_found_error, file_operation_error, glob_pattern_error, invalid_filename_error, Error,
    Result,
};

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
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

    // Helper function to create a temporary directory structure for testing
    fn create_test_directory_structure() -> (TempDir, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path().to_path_buf();

        // Create a directory structure for testing
        // root/
        //   ├── dir1/
        //   ├── dir2/
        //   └── test/
        //       ├── subdir1/
        //       ├── subdir2/
        //       └── subdir3/
        fs::create_dir(root_path.join("dir1")).unwrap();
        fs::create_dir(root_path.join("dir2")).unwrap();
        fs::create_dir(root_path.join("test")).unwrap();
        fs::create_dir(root_path.join("test").join("subdir1")).unwrap();
        fs::create_dir(root_path.join("test").join("subdir2")).unwrap();
        fs::create_dir(root_path.join("test").join("subdir3")).unwrap();

        (temp_dir, root_path)
    }

    #[test]
    fn test_first_transformation() {
        // Create a temporary directory structure
        let (_tempdir, root_path) = create_test_directory_structure();

        // Create a First transformative function with no arguments
        let first_function = FolderFunction::First { args: None };

        // Test get_dir with the root path
        let result = first_function.get_dir(&root_path);
        assert!(result.is_ok());

        // The result should be the first directory in the root path (alphabetically)
        let dir = result.unwrap();
        assert_eq!(dir.file_name().unwrap(), "dir1");

        // Create a First transformative function with arguments
        let first_function_with_args = FolderFunction::First {
            args: Some(vec!["test".to_string()]),
        };

        // Test get_dir with the root path and arguments
        let result = first_function_with_args.get_dir(&root_path);
        assert!(result.is_ok());

        // The result should be the first subdirectory in the test directory (alphabetically)
        let dir = result.unwrap();
        assert_eq!(dir.file_name().unwrap(), "subdir1");
    }

    #[test]
    fn test_last_transformation() {
        // Create a temporary directory structure
        let (_tempdir, root_path) = create_test_directory_structure();

        // Create a Last transformative function with no arguments
        let last_function = FolderFunction::Last { args: None };

        // Test get_dir with the root path
        let result = last_function.get_dir(&root_path);
        assert!(result.is_ok());

        // The result should be the last directory in the root path (alphabetically)
        let dir = result.unwrap();
        assert_eq!(dir.file_name().unwrap(), "test");

        // Create a Last transformative function with arguments
        let last_function_with_args = FolderFunction::Last {
            args: Some(vec!["test".to_string()]),
        };

        // Test get_dir with the root path and arguments
        let result = last_function_with_args.get_dir(&root_path);
        assert!(result.is_ok());

        // The result should be the last subdirectory in the test directory (alphabetically)
        let dir = result.unwrap();
        assert_eq!(dir.file_name().unwrap(), "subdir3");
    }

    #[test]
    fn test_construct_path() {
        // Create a temporary directory
        let (_tempdir, root_path) = create_test_directory_structure();

        // Create a First transformative function with no arguments
        let first_function = FolderFunction::First { args: None };

        // Test construct_path with the root path
        let path = first_function.construct_path(&root_path);

        // The path should be the root path with a wildcard appended
        assert_eq!(path, root_path.join("*"));

        // Create a First transformative function with arguments
        let first_function_with_args = FolderFunction::First {
            args: Some(vec!["test".to_string(), "subdir".to_string()]),
        };

        // Test construct_path with the root path and arguments
        let path = first_function_with_args.construct_path(&root_path);

        // The path should be the root path with the arguments appended, ending with a wildcard
        // e.g., root/test/subdir/*
        assert_eq!(path, root_path.join("test").join("subdir").join("*"));
    }

    #[test]
    fn test_get_result_based_on_transformation() {
        // Create a list of paths
        let paths = vec![
            PathBuf::from("path1"),
            PathBuf::from("path2"),
            PathBuf::from("path3"),
        ];

        // Create a First transformative function
        let first_function = FolderFunction::First { args: None };

        // Test get_result_based_on_transformation with the list of paths
        let result = first_function.get_result_based_on_transformation(paths.clone());
        assert!(result.is_ok());

        // The result should be the first path in the list
        let path = result.unwrap();
        assert_eq!(path, paths[0]);

        // Create a Last transformative function
        let last_function = FolderFunction::Last { args: None };

        // Test get_result_based_on_transformation with the list of paths
        let result = last_function.get_result_based_on_transformation(paths.clone());
        assert!(result.is_ok());

        // The result should be the last path in the list
        let path = result.unwrap();
        assert_eq!(path, paths[2]);
    }

    #[test]
    fn test_error_handling() {
        // Create a temporary directory
        let (_tempdir, root_path) = create_test_directory_structure();

        // Create a First transformative function with arguments that don't exist
        let first_function = FolderFunction::First {
            args: Some(vec!["nonexistent".to_string()]),
        };

        // Test get_dir with the root path and nonexistent arguments
        let result = first_function.get_dir(&root_path);
        assert!(result.is_err());

        // The error should indicate that the directory was not found
        let error = result.unwrap_err();
        assert!(error.to_string().contains("not found"));
    }
}
