//! Configuration module
//!
//! This module contains components for loading and validating configuration.

pub mod loader;
mod model;

pub use loader::{load_config, load_config_for_testing, prepare_rules, read_or_create};
pub use model::{Config, ConfigProcessor, FormatConversion, Rule, RulesList};
