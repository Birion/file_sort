//! File discovery module
//!
//! This module contains components for scanning directories and finding files.

pub mod content_analyser;
mod matcher;
mod scanner;

pub use content_analyser::{
    ConditionOperator, ContentAnalysis, ContentCondition, ContentProperty, FileMetadata,
};
pub use matcher::{match_file_against_rules, MatchResult};
pub use scanner::{scan_directory, FileInfo};
