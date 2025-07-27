use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use crate::constants::{APPLICATION, ORGANIZATION, QUALIFIER};
use crate::errors::{generic_error, pattern_extraction_error, pattern_matching_error, Result};
use chrono::TimeZone;
use chrono::Utc;
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};

/// Helper method to clean the pattern by removing angle brackets
pub fn clean_pattern(pattern: &str) -> Result<String> {
    static CLEAN_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"[<>]").map_err(|e| pattern_matching_error(e, r"[<>]"))
            .expect("Failed to compile regex pattern for clean_pattern - this is a static initialization error and should never happen")
    });
    Ok(CLEAN_RE.replace_all(pattern, "").to_string())
}

/// Helper method to extract content between angle brackets
pub fn extract_pattern(pattern: &str) -> Result<String> {
    static EXTRACT_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r".*<([^<>]*)>.*").map_err(|e| pattern_matching_error(e, r".*<([^<>]*)>.*"))
            .expect("Failed to compile regex pattern for extract_pattern - this is a static initialization error and should never happen")
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
        .ok_or_else(|| generic_error(&format!("Invalid timestamp: {timestamp}")))?
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_pattern() {
        // Test with angle brackets
        let result = clean_pattern("<pattern>").unwrap();
        assert_eq!(result, "pattern");

        // Test with multiple angle brackets
        let result = clean_pattern("<pat<ter>n>").unwrap();
        assert_eq!(result, "pattern");

        // Test with no angle brackets
        let result = clean_pattern("pattern").unwrap();
        assert_eq!(result, "pattern");

        // Test with an empty string
        let result = clean_pattern("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_extract_pattern() {
        // Test with angle brackets
        let result = extract_pattern("<pattern>").unwrap();
        assert_eq!(result, "pattern");

        // Test with no angle brackets
        let result = extract_pattern("pattern").unwrap();
        assert_eq!(result, "pattern");

        // Test with empty angle brackets
        let result = extract_pattern("<>").unwrap();
        assert_eq!(result, "");

        // Test with nested angle brackets (should only extract the innermost content)
        let result = extract_pattern("<outer<inner>>").unwrap();
        assert_eq!(result, "inner");
    }

    #[test]
    fn test_process_date() {
        // Test with a valid timestamp and splitter
        let mut destination = "1626912000_filename.txt".to_string();
        let fmt = "%Y-%m-%d";
        let splitter = "_";
        let merger = Some(" ".to_string());

        process_date(&mut destination, fmt, splitter, &merger).unwrap();
        assert_eq!(destination, "2021-07-22 filename.txt");

        // Test with different format
        let mut destination = "1626912000_filename.txt".to_string();
        let fmt = "%d/%m/%Y";
        let splitter = "_";
        let merger = Some("-".to_string());

        process_date(&mut destination, fmt, splitter, &merger).unwrap();
        assert_eq!(destination, "22/07/2021-filename.txt");

        // Test with invalid timestamp (should return an error)
        let mut destination = "invalid_filename.txt".to_string();
        let fmt = "%Y-%m-%d";
        let splitter = "_";
        let merger = Some(" ".to_string());

        let result = process_date(&mut destination, fmt, splitter, &merger);
        assert!(result.is_err());

        // Test with missing splitter (should return an error)
        let mut destination = "1626912000filename.txt".to_string();
        let fmt = "%Y-%m-%d";
        let splitter = "_";
        let merger = Some(" ".to_string());

        let result = process_date(&mut destination, fmt, splitter, &merger);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_pattern() {
        // Test with valid pattern and replacement
        let mut destination = "test_filename.txt".to_string();
        let pattern = "test";
        let replacement = Some("replaced".to_string());

        process_pattern(&mut destination, pattern, &replacement).unwrap();
        assert_eq!(destination, "replaced_filename.txt");

        // Test with regex pattern
        let mut destination = "test123_filename.txt".to_string();
        let pattern = r"test\d+";
        let replacement = Some("replaced".to_string());

        process_pattern(&mut destination, pattern, &replacement).unwrap();
        assert_eq!(destination, "replaced_filename.txt");

        // Test with no replacement (should not change the string)
        let mut destination = "test_filename.txt".to_string();
        let pattern = "test";
        let replacement = None;

        process_pattern(&mut destination, pattern, &replacement).unwrap();
        assert_eq!(destination, "test_filename.txt");

        // Test with invalid regex pattern (should return an error)
        let mut destination = "test_filename.txt".to_string();
        let pattern = "["; // Invalid regex pattern
        let replacement = Some("replaced".to_string());

        let result = process_pattern(&mut destination, pattern, &replacement);
        assert!(result.is_err());
    }
}
