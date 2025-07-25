//! File processing module
//!
//! This module contains components for processing files, including
//! file operations, path handling, pattern matching, and format conversion.

mod core;
mod file_operations;
mod format_conversion;
mod path_handling;
mod pattern_matching;

pub use core::{Processor, ProcessorBuilder};
pub use format_conversion::convert_file_format;