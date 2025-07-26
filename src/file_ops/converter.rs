//! File format conversion functionality
//!
//! This module provides functionality for converting files between different formats.
//! It supports image format conversion (using the `image` crate) and text encoding
//! conversion (using the `encoding_rs` crate).

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use image::ImageFormat;
use log::info;
use once_cell::sync::Lazy;
use tempfile::{NamedTempFile, TempPath};

use crate::config::{FormatConversion, Rule};
use crate::errors::generic_error;
use crate::path_gen::PathResult;

/// Supported image formats for conversion
pub const SUPPORTED_IMAGE_FORMATS: [&str; 7] = ["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff"];

/// Supported text encodings for conversion
pub const SUPPORTED_TEXT_ENCODINGS: [&str; 6] = [
    "utf-8",
    "utf-16",
    "utf-16le",
    "utf-16be",
    "iso-8859-1",
    "windows-1252",
];

/// Temporary path for file conversion
pub static TEMPORARY_PATH: Lazy<TempPath> =
    Lazy::new(|| NamedTempFile::new().unwrap().into_temp_path());

/// Result of converting a file format
#[derive(Debug, Clone)]
pub struct ConversionResult {
    /// The source path
    pub source_path: PathBuf,
    /// The target path
    pub target_path: PathBuf,
    /// The rule that was applied
    pub rule: Rule,
}

/// Converts a file from one format to another if needed
///
/// This function checks if the rule has a format conversion defined and applies it if needed.
///
/// # Arguments
/// * `path_result` - The result of generating a destination path
/// * `run_execution` - Whether to actually perform the file operations (false) or just simulate them (true)
///
/// # Returns
/// * `Result<ConversionResult>` - The result of the conversion or an error
///
/// # Errors
/// * Returns an error if the conversion fails
pub fn convert_file_format(
    path_result: &PathResult,
    run_execution: bool,
) -> Result<ConversionResult> {
    let source_path = &path_result.source_path;
    let target_path = &path_result.target_path;
    let rule = &path_result.rule;

    // Check if the rule has a format conversion defined
    if let Some(config_processor) = &rule.processors
        && let Some(format_conversion) = &config_processor.format_conversion
    {
        // Check if the source file format is supported
        let source_ext = source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if (SUPPORTED_IMAGE_FORMATS.contains(&format_conversion.source_format.as_str())
            && SUPPORTED_IMAGE_FORMATS.contains(&source_ext))
            || SUPPORTED_TEXT_ENCODINGS.contains(&format_conversion.source_format.as_str())
        {
            // Apply the format conversion
            let (converted_source, converted_target) =
                apply_conversion(source_path, target_path, format_conversion, run_execution)?;

            return Ok(ConversionResult {
                source_path: converted_source,
                target_path: converted_target,
                rule: rule.clone(),
            });
        }
    }

    // No conversion needed
    Ok(ConversionResult {
        source_path: source_path.to_path_buf(),
        target_path: target_path.to_path_buf(),
        rule: rule.clone(),
    })
}

/// Applies a format conversion to a file
///
/// This function determines the type of conversion to perform based on the
/// source and target formats specified in the `FormatConversion` struct.
///
/// # Arguments
/// * `source_path` - The path to the source file
/// * `target_path` - The path where the converted file will be saved
/// * `conversion` - The conversion configuration
/// * `run_execution` - Whether to actually perform the conversion (true) or just simulate it (false)
///
/// # Returns
/// * `Result<(PathBuf, PathBuf)>` - The source and target paths after conversion, or an error
///
/// # Errors
/// * Returns an error if the conversion fails
fn apply_conversion(
    source_path: &Path,
    target_path: &Path,
    conversion: &FormatConversion,
    run_execution: bool,
) -> Result<(PathBuf, PathBuf)> {
    // Determine the type of conversion based on the source and target formats
    match (
        conversion.source_format.as_str(),
        conversion.target_format.as_str(),
    ) {
        // Image format conversion
        (
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff",
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff",
        ) => convert_image_format(source_path, target_path, conversion, run_execution),
        // Text encoding conversion
        (
            "utf-8" | "utf-16" | "utf-16le" | "utf-16be" | "iso-8859-1" | "windows-1252",
            "utf-8" | "utf-16" | "utf-16le" | "utf-16be" | "iso-8859-1" | "windows-1252",
        ) => convert_text_encoding(source_path, target_path, conversion, run_execution),
        // Unsupported conversion
        _ => Err(generic_error(&format!(
            "Unsupported conversion from {} to {}",
            conversion.source_format, conversion.target_format
        ))
        .into()),
    }
}

/// Converts an image from one format to another
///
/// # Arguments
/// * `source_path` - The path to the source image
/// * `target_path` - The path where the converted image will be saved
/// * `conversion` - The conversion configuration
/// * `run_execution` - Whether to actually perform the conversion (true) or just simulate it (false)
///
/// # Returns
/// * `Result<(PathBuf, PathBuf)>` - The source and target paths after conversion, or an error
///
/// # Errors
/// * Returns an error if the image conversion fails
fn convert_image_format(
    source_path: &Path,
    target_path: &Path,
    conversion: &FormatConversion,
    run_execution: bool,
) -> Result<(PathBuf, PathBuf)> {
    let target_path = target_path.with_extension(conversion.target_format.as_str());

    if !run_execution {
        return Ok((source_path.to_path_buf(), target_path.to_path_buf()));
    }

    // Read the source image
    let img = image::open(source_path).map_err(|e| {
        generic_error(&format!(
            "Failed to open image {}: {}",
            source_path.display(),
            e
        ))
    })?;

    // Resize the image if requested
    let img = if let Some((width, height)) = conversion.resize {
        img.resize(width, height, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    // Determine the output format
    let format =
        ImageFormat::from_extension(conversion.target_format.as_str()).ok_or_else(|| {
            generic_error(&format!(
                "Unsupported image format: {}",
                conversion.target_format
            ))
        })?;

    // Save the image in the target format
    let mut output_file = fs::File::create(TEMPORARY_PATH.to_path_buf()).map_err(|e| {
        generic_error(&format!(
            "Failed to create file {}: {}",
            target_path.display(),
            e
        ))
    })?;

    img.write_to(&mut output_file, format).map_err(|e| {
        generic_error(&format!(
            "Failed to write image to {}: {}",
            target_path.display(),
            e
        ))
    })?;

    info!(
        "Converted image from {} to {}",
        source_path.display(),
        TEMPORARY_PATH.display()
    );

    // Remove the original source file if it exists
    fs::remove_file(source_path)?;

    Ok((TEMPORARY_PATH.to_path_buf(), target_path.to_path_buf()))
}

/// Converts a text file from one encoding to another
///
/// # Arguments
/// * `source_path` - The path to the source text file
/// * `target_path` - The path where the converted text file will be saved
/// * `conversion` - The conversion configuration
/// * `run_execution` - Whether to actually perform the conversion (true) or just simulate it (false)
///
/// # Returns
/// * `Result<(PathBuf, PathBuf)>` - The source and target paths after conversion, or an error
///
/// # Errors
/// * Returns an error if the text encoding conversion fails
fn convert_text_encoding(
    source_path: &Path,
    target_path: &Path,
    conversion: &FormatConversion,
    run_execution: bool,
) -> Result<(PathBuf, PathBuf)> {
    if !run_execution {
        return Ok((source_path.to_path_buf(), target_path.to_path_buf()));
    }

    // Read the source file
    let mut source_file = fs::File::open(source_path).map_err(|e| {
        generic_error(&format!(
            "Failed to open file {}: {}",
            source_path.display(),
            e
        ))
    })?;

    let mut content = Vec::new();
    source_file.read_to_end(&mut content).map_err(|e| {
        generic_error(&format!(
            "Failed to read file {}: {}",
            source_path.display(),
            e
        ))
    })?;

    // Determine the source encoding
    let source_encoding = match conversion.source_format.as_str() {
        "utf-8" => encoding_rs::UTF_8,
        "utf-16" | "utf-16le" => encoding_rs::UTF_16LE,
        "utf-16be" => encoding_rs::UTF_16BE,
        "iso-8859-1" => encoding_rs::WINDOWS_1252,
        "windows-1252" => encoding_rs::WINDOWS_1252,
        _ => {
            return Err(generic_error(&format!(
                "Unsupported text encoding: {}",
                conversion.source_format
            ))
            .into());
        }
    };

    // Determine the target encoding
    let target_encoding = match conversion.target_format.as_str() {
        "utf-8" => encoding_rs::UTF_8,
        "utf-16" | "utf-16le" => encoding_rs::UTF_16LE,
        "utf-16be" => encoding_rs::UTF_16BE,
        "iso-8859-1" => encoding_rs::WINDOWS_1252,
        "windows-1252" => encoding_rs::WINDOWS_1252,
        _ => {
            return Err(generic_error(&format!(
                "Unsupported text encoding: {}",
                conversion.target_format
            ))
            .into());
        }
    };

    // Decode the content using the source encoding
    let (decoded, _, _) = source_encoding.decode(&content);

    // Encode the content using the target encoding
    let (encoded, _, _) = target_encoding.encode(&decoded);

    // Create the target directory if it doesn't exist
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            generic_error(&format!(
                "Failed to create directory {}: {}",
                parent.display(),
                e
            ))
        })?;
    }

    // Write the encoded content to the target file
    let mut target_file = fs::File::create(TEMPORARY_PATH.to_path_buf()).map_err(|e| {
        generic_error(&format!(
            "Failed to create file {}: {}",
            target_path.display(),
            e
        ))
    })?;

    target_file.write_all(&encoded).map_err(|e| {
        generic_error(&format!(
            "Failed to write to file {}: {}",
            target_path.display(),
            e
        ))
    })?;

    Ok((source_path.to_path_buf(), TEMPORARY_PATH.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path_gen::PathResult;
    use tempfile::tempdir;

    #[test]
    fn test_convert_image_format() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test.png");
        let target_path = temp_dir.path().join("test.jpg");

        // Create a simple test image
        let img = image::ImageBuffer::from_fn(100, 100, |x, y| {
            if (x as i32 - 50).pow(2) + (y as i32 - 50).pow(2) < 2500 {
                image::Rgb::<u8>([255, 0, 0])
            } else {
                image::Rgb::<u8>([0, 0, 255])
            }
        });

        // Save the test image
        img.save(&source_path).unwrap();

        // Define the conversion
        let conversion = FormatConversion {
            source_format: "png".to_string(),
            target_format: "jpg".to_string(),
            resize: Some((50, 50)),
        };

        // Convert the image
        let result = convert_image_format(&source_path, &target_path, &conversion, true);
        assert!(result.is_ok());

        // Get the paths returned by the function
        let (new_source_path, new_target_path) = result.unwrap();

        // Copy the temporary file to the target path for testing
        fs::copy(&new_source_path, &new_target_path).unwrap();

        // Verify the converted image exists and has the correct format
        let converted_img = image::open(&new_target_path).unwrap();
        assert_eq!(converted_img.width(), 50);
        assert_eq!(converted_img.height(), 50);
    }

    #[test]
    fn test_convert_text_encoding() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test.txt");
        let target_path = temp_dir.path().join("test_utf16.txt");

        // Create a test text file in UTF-8
        let text = "Hello, world! 你好，世界！";
        fs::write(&source_path, text).unwrap();

        // Define the conversion
        let conversion = FormatConversion {
            source_format: "utf-8".to_string(),
            target_format: "utf-16".to_string(),
            resize: None,
        };

        // Convert the text file
        let result = convert_text_encoding(&source_path, &target_path, &conversion, true);
        assert!(result.is_ok());

        // Get the paths returned by the function
        let (_, new_target_path) = result.unwrap();

        // Verify that the converted file exists and has a non-zero size
        assert!(new_target_path.exists());
        assert!(fs::metadata(&new_target_path).unwrap().len() > 0);
    }

    #[test]
    fn test_format_conversion_integration() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test.txt");
        let target_path = temp_dir.path().join("target").join("output.txt");
        fs::create_dir_all(temp_dir.path().join("target")).unwrap();

        // Create a test text file in UTF-8
        let text = "Hello, world! 你好，世界！";
        fs::write(&source_path, text).unwrap();

        // Create a rule with format conversion
        let rule = Rule {
            title: "Test Format Conversion".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: None,
            function: None,
            processors: Some(crate::config::ConfigProcessor {
                splitter: None,
                merger: None,
                pattern: None,
                date_format: None,
                replacement: None,
                format_conversion: Some(FormatConversion {
                    source_format: "utf-8".to_string(),
                    target_format: "utf-16".to_string(),
                    resize: None,
                }),
            }),
            root: 0,
            copy: false,
            old_pattern: "pattern".to_string(),
            new_pattern: "pattern".to_string(),
        };

        // Create a PathResult to simulate the output from path_gen
        let path_result = PathResult {
            source_path: source_path.clone(),
            target_path: target_path.clone(),
            rule: rule.clone(),
        };

        // Call convert_file_format with the PathResult
        let result = convert_file_format(&path_result, true);
        assert!(result.is_ok());

        let conversion_result = result.unwrap();

        // Verify that the converted file exists and has a non-zero size
        assert!(conversion_result.target_path.exists());
        assert!(fs::metadata(&conversion_result.target_path).unwrap().len() > 0);
    }
}
