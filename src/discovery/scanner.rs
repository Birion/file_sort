//! Directory scanning functionality
//!
//! This module contains functions for scanning directories and finding files.

use std::fs::read_dir;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use log::debug;

use crate::discovery::content_analyser::analyse_file_content;
use crate::discovery::ContentAnalysis;
use crate::utils::is_hidden_file;

/// Information about a file found during scanning
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// The path to the file
    pub path: PathBuf,
    /// The filename of the file
    pub filename: String,
    /// Content analysis results (lazy-loaded)
    pub content_analysis: Option<ContentAnalysis>,
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

        Ok(FileInfo {
            path,
            filename,
            content_analysis: None, // Lazy-loaded when needed
        })
    }

    /// Ensures content analysis is loaded for the file
    ///
    /// This method lazily loads content analysis information if it hasn't been loaded yet.
    /// It's used when content-based rules need to be evaluated.
    ///
    /// # Returns
    /// * `Result<&ContentAnalysis>` - Reference to the content analysis or an error
    ///
    /// # Errors
    /// Returns an error if content analysis fails
    pub fn ensure_content_analysed(&mut self) -> Result<&ContentAnalysis> {
        if self.content_analysis.is_none() {
            debug!("Analysing content for file: {}", self.filename);
            self.content_analysis = Some(analyse_file_content(&self.path)?);
        }

        Ok(self.content_analysis.as_ref().unwrap())
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
