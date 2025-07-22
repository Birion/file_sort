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
    source: PathBuf,
    /// The target path where the file will be moved or copied to
    target: PathBuf,
}

impl Processor {
    /// Creates a new ProcessorBuilder for building a Processor instance
    ///
    /// # Arguments
    /// * `file` - The path to the file to be processed
    ///
    /// # Returns
    /// * `ProcessorBuilder` - A new ProcessorBuilder instance with the source set to the given file
    pub fn builder(file: &Path) -> ProcessorBuilder {
        ProcessorBuilder::new(file)
    }

    /// Gets a reference to the source path
    pub fn source(&self) -> &PathBuf {
        &self.source
    }

    /// Gets a reference to the target path
    pub fn target(&self) -> &PathBuf {
        &self.target
    }

    /// Gets a mutable reference to the target path
    pub(crate) fn target_mut(&mut self) -> &mut PathBuf {
        &mut self.target
    }

    /// Sets the target path
    pub(crate) fn set_target(&mut self, target: PathBuf) {
        self.target = target;
    }
}

/// Builder for creating Processor instances
///
/// This struct follows the builder pattern to provide a more readable
/// and flexible way to create Processor instances.
#[derive(Debug, Clone)]
pub struct ProcessorBuilder {
    /// The source path of the file being processed
    source: PathBuf,
    /// The target path where the file will be moved or copied to
    target: PathBuf,
}

impl ProcessorBuilder {
    /// Creates a new ProcessorBuilder instance for the given file
    ///
    /// # Arguments
    /// * `file` - The path to the file to be processed
    ///
    /// # Returns
    /// * `ProcessorBuilder` - A new ProcessorBuilder instance with the source set to the given file
    pub fn new(file: &Path) -> ProcessorBuilder {
        ProcessorBuilder {
            source: file.to_path_buf(),
            target: PathBuf::new(),
        }
    }

    /// Sets the source path
    ///
    /// # Arguments
    /// * `source` - The source path to set
    ///
    /// # Returns
    /// * `ProcessorBuilder` - The builder instance for method chaining
    pub fn source(mut self, source: PathBuf) -> ProcessorBuilder {
        self.source = source;
        self
    }

    /// Sets the target path
    ///
    /// # Arguments
    /// * `target` - The target path to set
    ///
    /// # Returns
    /// * `ProcessorBuilder` - The builder instance for method chaining
    pub fn target(mut self, target: PathBuf) -> ProcessorBuilder {
        self.target = target;
        self
    }

    /// Builds the Processor instance
    ///
    /// # Returns
    /// * `Processor` - The built Processor instance
    pub fn build(self) -> Processor {
        Processor {
            source: self.source,
            target: self.target,
        }
    }
}
