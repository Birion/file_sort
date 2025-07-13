use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use crate::constants::{APPLICATION, ORGANIZATION, QUALIFIER};
use crate::errors::{
    generic_error, path_operation_error, pattern_extraction_error, pattern_matching_error, Result,
};
use crate::processor::Processor;
use crate::rules::Rule;
use chrono::TimeZone;
use chrono::Utc;
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};

/// Helper method to clean the pattern by removing angle brackets
pub fn clean_pattern(pattern: &str) -> Result<String> {
    static CLEAN_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"[<>]").expect("Failed to compile regex pattern for clean_pattern")
    });
    Ok(CLEAN_RE.replace_all(pattern, "").to_string())
}

/// Helper method to extract content between angle brackets
pub fn extract_pattern(pattern: &str) -> Result<String> {
    static EXTRACT_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r".*<(.*)>.*").expect("Failed to compile regex pattern for extract_pattern")
    });

    let captures: Option<Captures> = EXTRACT_RE.captures(pattern);
    match captures {
        Some(c) => c
            .get(1)
            .ok_or_else(|| pattern_extraction_error(pattern, "No capture group found"))
            .map(|m| m.as_str().to_string()),
        None => Ok(pattern.to_string()),
    }
}

pub fn full_path(root: &Path, folder: &Path) -> PathBuf {
    root.join(folder)
}

/// Process date strings in the destination path
pub fn process_date(
    destination: &mut String,
    fmt: &str,
    splitter: &str,
    merger: &Option<String>,
) -> Result<()> {
    let parts: Vec<&str> = if splitter.contains('%') {
        let mut dt = Utc::now().date_naive();
        let mut _fmt = dt.format(splitter).to_string();
        while !destination.contains(&_fmt) {
            dt = dt
                .pred_opt()
                .ok_or_else(|| generic_error("Failed to decrement date"))?;
            _fmt = dt.format(splitter).to_string();
        }
        destination.split(&_fmt).collect()
    } else {
        destination.split(splitter).collect()
    };

    if parts.len() < 2 {
        return Err(generic_error(&format!(
            "Failed to split destination '{destination}' with splitter '{splitter}'"
        )));
    }

    let timestamp = parts[0]
        .parse::<i64>()
        .map_err(|e| generic_error(&format!("Failed to parse timestamp '{}': {}", parts[0], e)))?;

    let creation_date: String = Utc
        .timestamp_opt(timestamp, 0)
        .single()
        .ok_or_else(|| generic_error(&format!("Invalid timestamp: {}", timestamp)))?
        .format(fmt)
        .to_string();

    let merger_str = merger
        .as_ref()
        .ok_or_else(|| generic_error("Merger string is required but not provided"))?
        .as_str();

    *destination = [creation_date.as_str(), parts[1]].join(merger_str);

    Ok(())
}

/// Process a pattern in the destination string using regex replacement
pub fn process_pattern(
    destination: &mut String,
    pattern: &str,
    replacement: &Option<String>,
) -> Result<()> {
    let regex_pattern = Regex::new(pattern).map_err(|e| pattern_matching_error(e, pattern))?;

    *destination = match replacement {
        Some(replacement_value) => regex_pattern
            .replace(destination.as_str(), replacement_value)
            .to_string(),
        None => destination.to_string(),
    };

    Ok(())
}

/// Generate the target path for a file based on the rule and processor
pub(crate) fn generate_target(processor: &Processor, rule: &Rule, root: &Path) -> Result<PathBuf> {
    match &rule.function {
        None => processor.make_destination(&rule.new_pattern, Some(root), rule),
        Some(func) => {
            let temporary_root = processor.make_destination(&rule.new_pattern, None, rule)?;
            let parent = temporary_root.parent().ok_or_else(|| {
                path_operation_error(temporary_root.clone(), "get parent directory")
            })?;

            let directory = func.get_dir(parent)?;
            processor.make_destination(&rule.new_pattern, Some(&directory), rule)
        }
    }
}

pub(crate) fn find_project_folder() -> Result<ProjectDirs> {
    let folder = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| generic_error("Failed to determine project directories"))?;

    if !folder.config_dir().exists() {
        create_dir_all(folder.config_dir())?;
    }
    Ok(folder)
}

#[cfg(unix)]
pub(crate) fn is_hidden_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map_or(false, |name| name.starts_with('.'))
}

#[cfg(windows)]
pub(crate) fn is_hidden_file(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;

    if let Ok(metadata) = path.metadata() {
        metadata.file_attributes() & 0x2 != 0 // FILE_ATTRIBUTE_HIDDEN
    } else {
        false
    }
}
