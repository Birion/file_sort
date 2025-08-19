//! Workflow context
//!
//! This module defines the context passed between workflow steps.

use std::path::PathBuf;

use crate::config::Config;

/// Represents a planned file operation for dry-run mode
#[derive(Debug, Clone)]
pub struct PlannedOperation {
    /// The source path of the file
    pub source: PathBuf,
    /// The destination path of the file
    pub destination: PathBuf,
    /// The type of operation (copy, move, convert)
    pub operation_type: OperationType,
    /// The rule title that matched this file
    pub rule_title: String,
}

/// Type of file operation
#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    /// Copy operation
    Copy,
    /// Move operation
    Move,
    /// Format conversion operation
    Convert(String, String), // from_format, to_format
}

/// Context for the workflow
///
/// This struct contains the state that is passed between workflow steps.
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    /// The configuration
    pub config: Config,
    /// Whether to actually perform file operations (true) or just simulate them (false)
    pub dry_run: bool,
    /// Statistics about the processing
    pub stats: WorkflowStats,
    /// Planned operations for dry-run mode
    pub planned_operations: Vec<PlannedOperation>,
}

/// Statistics about the workflow
///
/// This struct contains statistics about the processing.
#[derive(Debug, Clone, Default)]
pub struct WorkflowStats {
    /// Number of files processed
    pub files_processed: usize,
    /// Number of files matched
    pub files_matched: usize,
    /// Number of files moved
    pub files_moved: usize,
    /// Number of files copied
    pub files_copied: usize,
    /// Number of files converted
    pub files_converted: usize,
    /// Number of errors
    pub errors: usize,
}

impl WorkflowContext {
    /// Creates a new workflow context
    ///
    /// # Arguments
    /// * `config` - The configuration
    /// * `dry_run` - Whether to actually perform file operations (false) or just simulate them (true)
    ///
    /// # Returns
    /// * `WorkflowContext` - The new workflow context
    pub fn new(config: Config, dry_run: bool) -> Self {
        WorkflowContext {
            config,
            dry_run,
            stats: WorkflowStats::default(),
            planned_operations: Vec::new(),
        }
    }
    
    /// Adds a planned operation to the context
    ///
    /// # Arguments
    /// * `operation` - The planned operation to add
    pub fn add_planned_operation(&mut self, operation: PlannedOperation) {
        self.planned_operations.push(operation);
    }

    /// Increments the number of files processed
    pub fn increment_files_processed(&mut self) {
        self.stats.files_processed += 1;
    }

    /// Increments the number of files matched
    pub fn increment_files_matched(&mut self) {
        self.stats.files_matched += 1;
    }

    /// Increments the number of files moved
    pub fn increment_files_moved(&mut self) {
        self.stats.files_moved += 1;
    }

    /// Increments the number of files copied
    pub fn increment_files_copied(&mut self) {
        self.stats.files_copied += 1;
    }

    /// Increments the number of files converted
    pub fn increment_files_converted(&mut self) {
        self.stats.files_converted += 1;
    }

    /// Increments the number of errors
    pub fn increment_errors(&mut self) {
        self.stats.errors += 1;
    }
}
