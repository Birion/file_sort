//! Path generation module
//!
//! This module contains components for generating destination paths and applying transformative functions.

mod pattern;
mod transformer;

pub use pattern::{PathResult, generate_destination_path};
pub use transformer::{TransformResult, apply_transformative_function, FolderFunction, ArgumentList};
