//! File operations module
//!
//! This module contains components for file operations and format conversion.

mod actions;
mod converter;

pub use actions::{FileActionResult, perform_file_action};
pub use converter::{ConversionResult, convert_file_format};
