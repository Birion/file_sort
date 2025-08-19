//! File content analysis functionality
//!
//! This module contains functions for analysing file content and metadata.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::time::SystemTime;

use anyhow::{anyhow, Result};
use chrono;
use log::debug;
use serde::{Deserialize, Serialize};

/// Represents file metadata information
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File size in bytes
    pub size: u64,
    /// Last modified time
    pub modified: SystemTime,
    /// File creation time
    pub created: SystemTime,
    /// MIME type of the file
    pub mime_type: String,
}

/// Represents content analysis results
#[derive(Debug, Clone)]
pub struct ContentAnalysis {
    /// File metadata
    pub metadata: FileMetadata,
    /// Text content preview (first few lines or bytes)
    pub text_preview: Option<String>,
    /// Whether the file is a text file
    pub is_text: bool,
    /// Whether the file is a binary file
    pub is_binary: bool,
}

/// Condition operator for content-based rules
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum ConditionOperator {
    /// Equal to
    #[serde(alias = "eq")]
    Equal,
    /// Not equal to
    #[serde(alias = "ne")]
    NotEqual,
    /// Greater than
    #[serde(alias = "gt")]
    GreaterThan,
    /// Less than
    #[serde(alias = "lt")]
    LessThan,
    /// Greater than or equal to
    #[serde(alias = "ge")]
    GreaterThanOrEqual,
    /// Less than or equal to
    #[serde(alias = "le")]
    LessThanOrEqual,
    /// Contains
    #[serde(alias = "contains")]
    Contains,
    /// Starts with
    #[serde(alias = "startswith")]
    StartsWith,
    /// Ends with
    #[serde(alias = "endswith")]
    EndsWith,
    /// Matches regex
    #[serde(alias = "matches")]
    Matches,
}

/// Property to check in content-based rules
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum ContentProperty {
    /// File size in bytes
    #[serde(alias = "size")]
    Size,
    /// Last modified time
    #[serde(alias = "modified")]
    Modified,
    /// File creation time
    #[serde(alias = "created")]
    Created,
    /// MIME type of the file
    #[serde(alias = "mime_type")]
    MimeType,
    /// Text content of the file
    #[serde(alias = "content")]
    Content,
    /// Whether the file is a text file
    #[serde(alias = "is_text")]
    IsText,
    /// Whether the file is a binary file
    #[serde(alias = "is_binary")]
    IsBinary,
}

/// Represents a content-based condition for file matching
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ContentCondition {
    /// Property to check
    pub property: ContentProperty,
    /// Operator to use for comparison
    pub operator: ConditionOperator,
    /// Value to compare against
    pub value: String,
}

/// Gets file metadata
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// * `Result<FileMetadata>` - File metadata or an error
///
/// # Errors
/// Returns an error if the file metadata cannot be read
pub fn get_file_metadata(path: &Path) -> Result<FileMetadata> {
    let metadata = fs::metadata(path)?;

    // Get file times, defaulting to UNIX_EPOCH if not available
    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let created = metadata.created().unwrap_or(SystemTime::UNIX_EPOCH);

    // Get file size
    let size = metadata.len();

    // Determine MIME type (simple implementation)
    let mime_type = match path.extension().and_then(|e| e.to_str()) {
        Some("txt") => "text/plain".to_string(),
        Some("html") | Some("htm") => "text/html".to_string(),
        Some("css") => "text/css".to_string(),
        Some("js") => "application/javascript".to_string(),
        Some("json") => "application/json".to_string(),
        Some("xml") => "application/xml".to_string(),
        Some("pdf") => "application/pdf".to_string(),
        Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
        Some("png") => "image/png".to_string(),
        Some("gif") => "image/gif".to_string(),
        Some("svg") => "image/svg+xml".to_string(),
        Some("mp3") => "audio/mpeg".to_string(),
        Some("mp4") => "video/mp4".to_string(),
        Some("zip") => "application/zip".to_string(),
        Some("doc") | Some("docx") => "application/msword".to_string(),
        Some("xls") | Some("xlsx") => "application/vnd.ms-excel".to_string(),
        Some("ppt") | Some("pptx") => "application/vnd.ms-powerpoint".to_string(),
        _ => "application/octet-stream".to_string(),
    };

    Ok(FileMetadata {
        size,
        modified,
        created,
        mime_type,
    })
}

/// Checks if a file is likely to be a text file
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// * `Result<bool>` - True if the file is likely to be a text file, false otherwise
///
/// # Errors
/// Returns an error if the file cannot be read
fn is_text_file(path: &Path) -> Result<bool> {
    // Open the file
    let mut file = File::open(path)?;

    // Read the first 1024 bytes
    let mut buffer = [0; 1024];
    let bytes_read = file.read(&mut buffer)?;

    // If we couldn't read any bytes, assume it's not a text file
    if bytes_read == 0 {
        return Ok(false);
    }

    // Check for null bytes, which are common in binary files but rare in text files
    for &byte in &buffer[0..bytes_read] {
        if byte == 0 {
            return Ok(false);
        }
    }

    // If we didn't find any null bytes, assume it's a text file
    Ok(true)
}

/// Gets a preview of the file content
///
/// # Arguments
/// * `path` - Path to the file
/// * `max_bytes` - Maximum number of bytes to read
///
/// # Returns
/// * `Result<String>` - Preview of the file content or an error
///
/// # Errors
/// Returns an error if the file cannot be read
fn get_content_preview(path: &Path, max_bytes: usize) -> Result<String> {
    // Open the file
    let mut file = File::open(path)?;

    // Read up to max_bytes
    let mut buffer = vec![0; max_bytes];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    // Convert to string, replacing invalid UTF-8 sequences
    let preview = String::from_utf8_lossy(&buffer).to_string();

    Ok(preview)
}

/// Analyses file content
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// * `Result<ContentAnalysis>` - Content analysis results or an error
///
/// # Errors
/// Returns an error if the file cannot be analysed
pub fn analyse_file_content(path: &Path) -> Result<ContentAnalysis> {
    debug!("Analysing file content: {}", path.display());

    // Get file metadata
    let metadata = get_file_metadata(path)?;

    // Check if the file is a text file
    let is_text = is_text_file(path)?;
    let is_binary = !is_text;

    // Get content preview if it's a text file
    let text_preview = if is_text {
        get_content_preview(path, 1024).ok()
    } else {
        None
    };

    Ok(ContentAnalysis {
        metadata,
        text_preview,
        is_text,
        is_binary,
    })
}

/// Evaluates a content condition against file content analysis
///
/// # Arguments
/// * `condition` - The condition to evaluate
/// * `analysis` - The content analysis results
///
/// # Returns
/// * `Result<bool>` - True if the condition is met, false otherwise
///
/// # Errors
/// Returns an error if the condition cannot be evaluated
pub fn evaluate_condition(
    condition: &ContentCondition,
    analysis: &ContentAnalysis,
) -> Result<bool> {
    match condition.property {
        ContentProperty::Size => {
            let size = analysis.metadata.size;
            let value = condition
                .value
                .parse::<u64>()
                .map_err(|_| anyhow!("Invalid size value: {}", condition.value))?;

            match condition.operator {
                ConditionOperator::Equal => Ok(size == value),
                ConditionOperator::NotEqual => Ok(size != value),
                ConditionOperator::GreaterThan => Ok(size > value),
                ConditionOperator::LessThan => Ok(size < value),
                ConditionOperator::GreaterThanOrEqual => Ok(size >= value),
                ConditionOperator::LessThanOrEqual => Ok(size <= value),
                _ => Err(anyhow!(
                    "Invalid operator for size comparison: {:?}",
                    condition.operator
                )),
            }
        }
        ContentProperty::MimeType => {
            let mime_type = &analysis.metadata.mime_type;

            match condition.operator {
                ConditionOperator::Equal => Ok(mime_type == &condition.value),
                ConditionOperator::NotEqual => Ok(mime_type != &condition.value),
                ConditionOperator::Contains => Ok(mime_type.contains(&condition.value)),
                ConditionOperator::StartsWith => Ok(mime_type.starts_with(&condition.value)),
                ConditionOperator::EndsWith => Ok(mime_type.ends_with(&condition.value)),
                _ => Err(anyhow!(
                    "Invalid operator for mime type comparison: {:?}",
                    condition.operator
                )),
            }
        }
        ContentProperty::IsText => {
            let is_text = analysis.is_text;
            let value = condition
                .value
                .parse::<bool>()
                .map_err(|_| anyhow!("Invalid boolean value: {}", condition.value))?;

            match condition.operator {
                ConditionOperator::Equal => Ok(is_text == value),
                ConditionOperator::NotEqual => Ok(is_text != value),
                _ => Err(anyhow!(
                    "Invalid operator for is_text comparison: {:?}",
                    condition.operator
                )),
            }
        }
        ContentProperty::IsBinary => {
            let is_binary = analysis.is_binary;
            let value = condition
                .value
                .parse::<bool>()
                .map_err(|_| anyhow!("Invalid boolean value: {}", condition.value))?;

            match condition.operator {
                ConditionOperator::Equal => Ok(is_binary == value),
                ConditionOperator::NotEqual => Ok(is_binary != value),
                _ => Err(anyhow!(
                    "Invalid operator for is_binary comparison: {:?}",
                    condition.operator
                )),
            }
        }
        ContentProperty::Content => {
            if let Some(preview) = &analysis.text_preview {
                match condition.operator {
                    ConditionOperator::Contains => Ok(preview.contains(&condition.value)),
                    ConditionOperator::StartsWith => Ok(preview.starts_with(&condition.value)),
                    ConditionOperator::EndsWith => Ok(preview.ends_with(&condition.value)),
                    ConditionOperator::Matches => {
                        use regex::Regex;
                        let regex = Regex::new(&condition.value)?;
                        Ok(regex.is_match(preview))
                    }
                    _ => Err(anyhow!(
                        "Invalid operator for content comparison: {:?}",
                        condition.operator
                    )),
                }
            } else {
                // If there's no text preview, the condition is not met
                Ok(false)
            }
        }
        ContentProperty::Modified => {
            let modified = analysis.metadata.modified;

            // Parse the condition value as a date string (ISO 8601 format)
            let value_date =
                chrono::DateTime::parse_from_rfc3339(&condition.value).map_err(|_| {
                    anyhow!(
                        "Invalid date format: {}. Use ISO 8601 format (e.g., 2025-08-07T07:23:00Z)",
                        condition.value
                    )
                })?;

            // Convert SystemTime to chrono::DateTime
            let file_time = chrono::DateTime::<chrono::Utc>::from(modified);

            match condition.operator {
                ConditionOperator::Equal => Ok(file_time == value_date),
                ConditionOperator::NotEqual => Ok(file_time != value_date),
                ConditionOperator::GreaterThan => Ok(file_time > value_date),
                ConditionOperator::LessThan => Ok(file_time < value_date),
                ConditionOperator::GreaterThanOrEqual => Ok(file_time >= value_date),
                ConditionOperator::LessThanOrEqual => Ok(file_time <= value_date),
                _ => Err(anyhow!(
                    "Invalid operator for date comparison: {:?}",
                    condition.operator
                )),
            }
        }
        ContentProperty::Created => {
            let created = analysis.metadata.created;

            // Parse the condition value as a date string (ISO 8601 format)
            let value_date =
                chrono::DateTime::parse_from_rfc3339(&condition.value).map_err(|_| {
                    anyhow!(
                        "Invalid date format: {}. Use ISO 8601 format (e.g., 2025-08-07T07:23:00Z)",
                        condition.value
                    )
                })?;

            // Convert SystemTime to chrono::DateTime
            let file_time = chrono::DateTime::<chrono::Utc>::from(created);

            match condition.operator {
                ConditionOperator::Equal => Ok(file_time == value_date),
                ConditionOperator::NotEqual => Ok(file_time != value_date),
                ConditionOperator::GreaterThan => Ok(file_time > value_date),
                ConditionOperator::LessThan => Ok(file_time < value_date),
                ConditionOperator::GreaterThanOrEqual => Ok(file_time >= value_date),
                ConditionOperator::LessThanOrEqual => Ok(file_time <= value_date),
                _ => Err(anyhow!(
                    "Invalid operator for date comparison: {:?}",
                    condition.operator
                )),
            }
        }
    }
}
