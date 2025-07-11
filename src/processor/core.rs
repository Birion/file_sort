//! Core processor functionality
//!
//! This module contains the core Processor struct and its basic methods.

use std::path::{Path, PathBuf};

/// Handles file processing operations for sorting files
///
/// The Processor struct is responsible for managing the source and target paths
/// of files being processed and performing operations like copying, moving,
/// and pattern matching on these files.
#[derive(Debug, Clone)]
pub struct Processor {
    /// The source path of the file being processed
    pub(crate) source: PathBuf,
    /// The target path where the file will be moved or copied to
    pub(crate) target: PathBuf,
}

impl Processor {
    /// Creates a new Processor instance for the given file
    ///
    /// # Arguments
    /// * `file` - The path to the file to be processed
    ///
    /// # Returns
    /// * `Processor` - A new Processor instance with the source set to the given file
    pub(crate) fn new(file: &Path) -> Processor {
        Processor {
            source: file.to_path_buf(),
            target: PathBuf::new(),
        }
    }
}