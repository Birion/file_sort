//! File operation functionality
//!
//! This module contains methods for performing file operations like
//! copying, moving, and creating directories.

use crate::errors::{file_operation_error, Result};
use crate::utils::full_path;
use fs_extra::file::{copy, move_file, CopyOptions};
use std::fs::create_dir_all;
use std::path::Path;

use super::core::Processor;

impl Processor {
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
        let options = CopyOptions::new();
        if is_copy_operation {
            copy(&self.source, &self.target, &options).map_err(|e| {
                file_operation_error(std::io::Error::other(e), self.source.clone(), "copy")
            })?;
        }
        if is_rename_operation {
            move_file(&self.source, &self.target, &options).map_err(|e| {
                file_operation_error(std::io::Error::other(e), self.source.clone(), "move")
            })?;
        }
        Ok(())
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
}
