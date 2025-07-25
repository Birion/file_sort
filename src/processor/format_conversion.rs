//! File format conversion functionality
//!
//! This module provides functionality for converting files between different formats.
//! It supports image format conversion (using the `image` crate) and text encoding
//! conversion (using the `encoding_rs` crate).

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::errors::{generic_error, Result};
use crate::rules::FormatConversion;
use encoding_rs::UTF_8;
use image::ImageFormat;
use log::info;
use once_cell::sync::Lazy;
use tempfile::{NamedTempFile, TempPath};

pub const SUPPORTED_IMAGE_FORMATS: [&str; 7] = ["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff"];
pub const SUPPORTED_TEXT_ENCODINGS: [&str; 6] = [
    "utf-8",
    "utf-16",
    "utf-16le",
    "utf-16be",
    "iso-8859-1",
    "windows-1252",
];
pub static TEMPORARY_PATH: Lazy<TempPath> =
    Lazy::new(|| NamedTempFile::new().unwrap().into_temp_path());

/// Converts a file from one format to another
///
/// This function determines the type of conversion to perform based on the
/// source and target formats specified in the `FormatConversion` struct.
///
/// # Arguments
/// * `source_path` - The path to the source file
/// * `target_path` - The path where the converted file will be saved
/// * `conversion` - The conversion configuration
///
/// # Returns
/// * `Result<PathBuf>` - The path to the converted file, or an error
///
/// # Errors
/// * Returns an error if the conversion fails
pub fn convert_file_format(
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
        ))),
    }
}

/// Converts an image from one format to another
///
/// # Arguments
/// * `source_path` - The path to the source image
/// * `target_path` - The path where the converted image will be saved
/// * `conversion` - The conversion configuration
///
/// # Returns
/// * `Result<(PathBuf, PathBuf)>` - The path to the converted image, or an error
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
///
/// # Returns
/// * `Result<PathBuf>` - The path to the converted text file, or an error
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
    let target_path = source_path.with_extension("tmp");
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
        "utf-8" => UTF_8,
        "utf-16" | "utf-16le" => encoding_rs::UTF_16LE,
        "utf-16be" => encoding_rs::UTF_16BE,
        "iso-8859-1" => encoding_rs::WINDOWS_1252,
        "windows-1252" => encoding_rs::WINDOWS_1252,
        _ => {
            return Err(generic_error(&format!(
                "Unsupported text encoding: {}",
                conversion.source_format
            )));
        }
    };

    // Determine the target encoding
    let target_encoding = match conversion.target_format.as_str() {
        "utf-8" => UTF_8,
        "utf-16" | "utf-16le" => encoding_rs::UTF_16LE,
        "utf-16be" => encoding_rs::UTF_16BE,
        "iso-8859-1" => encoding_rs::WINDOWS_1252,
        "windows-1252" => encoding_rs::WINDOWS_1252,
        _ => {
            return Err(generic_error(&format!(
                "Unsupported text encoding: {}",
                conversion.target_format
            )));
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

        // Verify the converted image exists and has the correct format
        let converted_img = image::open(&target_path).unwrap();
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

        // Read the converted file
        let mut file = fs::File::open(&target_path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        // Decode the content using UTF-16LE
        let (decoded, _, _) = encoding_rs::UTF_16LE.decode(&buffer);
        assert_eq!(decoded, text);
    }
}
