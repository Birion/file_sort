//! File processing module
//!
//! This module contains components for processing files, including
//! file operations, path handling, and pattern matching.

mod core;
mod file_operations;
mod path_handling;
mod pattern_matching;

pub use core::{Processor, ProcessorBuilder};