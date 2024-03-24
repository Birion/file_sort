use std::path::{Path, PathBuf};

use chrono::TimeZone;
use chrono::Utc;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};

use crate::{Processor, Rule};

// Helper method to clean pattern
pub fn clean_pattern(pattern: &str) -> anyhow::Result<String> {
    static CLEAN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[<>]").unwrap());
    Ok(CLEAN_RE.replace_all(pattern, "").to_string())
}

// Helper method to extract pattern
pub fn extract_pattern(pattern: &str) -> anyhow::Result<String> {
    static EXTRACT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r".*<(.*)>.*").unwrap());
    let captures: Option<Captures> = EXTRACT_RE.captures(pattern);
    match captures {
        Some(c) => Ok(c.get(1).unwrap().as_str().to_string()),
        None => Ok(pattern.to_string()),
    }
}

pub fn full_path(root: &Path, folder: &Path) -> PathBuf {
    root.join(folder)
}

pub fn process_date(destination: &mut String, fmt: &str, splitter: &str, merger: &Option<String>) -> anyhow::Result<()> {
    let parts: Vec<&str> = if splitter.contains('%') {
        let mut dt = Utc::now().date_naive();
        let mut _fmt = dt.format(splitter).to_string();
        while !destination.contains(&_fmt) {
            dt = dt.pred_opt().unwrap();
            _fmt = dt.format(splitter).to_string();
        }
        destination.split(&_fmt).collect()
    } else {
        destination.split(splitter).collect()
    };
    let creation_date: String = Utc
        .timestamp_opt(parts[0].parse()?, 0)
        .unwrap()
        .format(fmt)
        .to_string();
    *destination = [creation_date.as_str(), parts[1]]
        .join(merger.as_ref().unwrap().as_str());

    Ok(())
}

pub fn process_pattern(destination: &mut String, pattern: &str, replacement: &Option<String>) -> anyhow::Result<()> {
    let pattern = Regex::new(pattern)?;
    *destination = match replacement {
        Some(replacement_value) => pattern.replace(destination.as_str(), replacement_value).to_string(),
        None => destination.to_string(),
    };

    Ok(())
}

pub(crate) fn generate_target(processor: &Processor, rule: &Rule, root: &Path) -> anyhow::Result<PathBuf> {
    match &rule.function {
        None => processor.make_destination(&rule.new_pattern, Some(root), rule),
        Some(func) => match func {
            &_ => {
                let temporary_root = processor.make_destination(&rule.new_pattern, None, rule)?;
                let directory = func.get_dir(temporary_root.parent().unwrap())?;
                processor.make_destination(&rule.new_pattern, Some(&directory), rule)
            }
        },
    }
}
