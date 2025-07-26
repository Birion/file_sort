//! File operation functionality
//!
//! This module contains functions for performing file operations like
//! copying, moving, and creating directories.

use std::fs::create_dir_all;
use std::path::PathBuf;

use anyhow::Result;
use fs_extra::file::{CopyOptions, copy, move_file};
use log::debug;

use crate::errors::file_operation_error;
use crate::file_ops::converter::ConversionResult;

/// Result of performing a file action
#[derive(Debug, Clone)]
pub struct FileActionResult {
    /// The source path
    pub source_path: PathBuf,
    /// The target path
    pub target_path: PathBuf,
    /// Whether the operation was successful
    pub success: bool,
}

/// Performs a file action (copy or move)
///
/// This function performs a file action (copy or move) based on the rule's copy flag.
///
/// # Arguments
/// * `conversion_result` - The result of converting a file format
/// * `run_execution` - Whether to actually perform the file operations (true) or just simulate them (false)
///
/// # Returns
/// * `Result<FileActionResult>` - The result of the file action or an error
///
/// # Errors
/// * Returns an error if the file action fails
pub fn perform_file_action(
    conversion_result: &ConversionResult,
    run_execution: bool,
) -> Result<FileActionResult> {
    let source_path = &conversion_result.source_path;
    let target_path = &conversion_result.target_path;
    let rule = &conversion_result.rule;

    // Create the target directory if it doesn't exist
    if let Some(parent) = target_path.parent() {
        create_dir_all(parent)
            .map_err(|e| file_operation_error(e, parent.to_path_buf(), "create directory"))?;
    }

    if !run_execution {
        // Simulation mode, don't actually perform the file operation
        debug!(
            "Simulating file action: {} -> {}",
            source_path.display(),
            target_path.display()
        );
        return Ok(FileActionResult {
            source_path: source_path.to_path_buf(),
            target_path: target_path.to_path_buf(),
            success: true,
        });
    }

    // Determine whether to copy or move the file
    let is_copy_operation = rule.copy;
    let options = CopyOptions::new().overwrite(true);

    if is_copy_operation {
        // Copy the file
        debug!(
            "Copying file: {} -> {}",
            source_path.display(),
            target_path.display()
        );
        copy(source_path, target_path, &options).map_err(|e| {
            file_operation_error(std::io::Error::other(e), source_path.clone(), "copy")
        })?;
    } else {
        // Move the file
        debug!(
            "Moving file: {} -> {}",
            source_path.display(),
            target_path.display()
        );
        move_file(source_path, target_path, &options).map_err(|e| {
            file_operation_error(std::io::Error::other(e), source_path.clone(), "move")
        })?;
    }

    Ok(FileActionResult {
        source_path: source_path.to_path_buf(),
        target_path: target_path.to_path_buf(),
        success: true,
    })
}
