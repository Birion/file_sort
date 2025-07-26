//! Configuration module
//!
//! This module contains components for loading and validating configuration.

mod loader;
mod model;

pub use loader::{load_config, prepare_rules, read_or_create};
pub use model::{Config, ConfigProcessor, FormatConversion, Rule, RulesList};
