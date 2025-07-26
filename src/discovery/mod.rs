//! File discovery module
//!
//! This module contains components for scanning directories and finding files.

mod matcher;
mod scanner;

pub use matcher::{MatchResult, match_file_against_rules};
pub use scanner::{FileInfo, scan_directory};
