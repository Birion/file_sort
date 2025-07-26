//! Workflow module
//!
//! This module contains components for orchestrating the workflow steps.

mod context;
mod engine;

pub use context::WorkflowContext;
pub use engine::{ProcessingOptions, process_files};
