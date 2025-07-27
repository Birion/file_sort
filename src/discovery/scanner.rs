//! Directory scanning functionality
//!
//! This module contains functions for scanning directories and finding files.

use std::fs::read_dir;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use log::debug;

use crate::utils::is_hidden_file;

/// Information about a file found during scanning
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// The path to the file
    pub path: PathBuf,
    /// The filename of the file
    pub filename: String,
}

impl FileInfo {
    /// Creates a new FileInfo from a path
    ///
    /// # Arguments
    /// * `path` - The path to the file
    ///
    /// # Returns
    /// * `Result<FileInfo>` - The file information or an error
    ///
    /// # Errors
    /// Returns an error if the filename cannot be extracted or converted to a string
    pub fn new(path: PathBuf) -> Result<Self> {
        let filename = path
            .file_name()
            .ok_or_else(|| anyhow!("Failed to get filename from path: {}", path.display()))?
            .to_str()
            .ok_or_else(|| anyhow!("Invalid filename: {}", path.display()))?
            .to_string();

        Ok(FileInfo { path, filename })
    }
}

/// Scans a directory for files
///
/// This function scans the specified directory and returns a list of files found.
/// It filters out hidden files and directories.
///
/// # Arguments
/// * `directory` - The directory to scan
///
/// # Returns
/// * `Result<Vec<FileInfo>>` - A list of files found or an error
///
/// # Errors
/// Returns an error if the directory cannot be read
pub fn scan_directory(directory: &Path) -> Result<Vec<FileInfo>> {
    debug!("Scanning directory: {}", directory.display());

    let files: Vec<FileInfo> = read_dir(directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| !is_hidden_file(path))
        .filter(|path| path.is_file())
        .filter_map(|path| FileInfo::new(path).ok())
        .collect();

    debug!("Found {} files in directory", files.len());

    Ok(files)
}
