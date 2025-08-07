//! Configuration module
//!
//! This module contains components for loading and validating configuration.

pub mod deserializer;
mod model;
pub mod serializer;
pub mod wizard;

pub use deserializer::{load_config, load_config_for_testing, prepare_rules, read_or_create};
pub use model::{Config, ConfigProcessor, FormatConversion, Rule, RulesList};
pub use serializer::{
    serialize_optional_pathbuf_to_array, serialize_pathbuf_to_array,
    serialize_pathbuf_vec_to_arrays, serialize_rules,
};
pub use wizard::create_config_with_wizard;
