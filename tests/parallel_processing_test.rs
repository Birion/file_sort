use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

use file_sort::workflow::{process_files, ProcessingOptions};
use tempfile::tempdir;

#[test]
fn test_parallel_processing() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let download_dir = temp_dir.path().join("download");
    let target_dir = temp_dir.path().join("target");

    // Create the download and target directories
    create_dir_all(&download_dir).expect("Failed to create download directory");
    create_dir_all(&target_dir).expect("Failed to create target directory");

    // Create test files in the download directory
    for i in 0..10 {
        let file_path = download_dir.join(format!("test_file_{i}.txt"));
        let mut file = File::create(&file_path).expect("Failed to create test file");
        writeln!(file, "Test content for file {i}").expect("Failed to write to test file");
    }

    // Create a test configuration file
    let config_path = temp_dir.path().join("../config.yaml");
    let config_content = format!(
        r#"
download:
  - {}
root:
  - - "{}"
rules:
  - title: Test Rule
    pattern: test_file_<num>.txt
    directory: "{}/sorted"
    "#,
        download_dir.to_string_lossy().replace('\\', "\\\\"),
        target_dir.to_string_lossy().replace('\\', "\\\\"),
        target_dir.to_string_lossy().replace('\\', "\\\\")
    );

    let mut config_file = File::create(&config_path).expect("Failed to create config file");
    config_file
        .write_all(config_content.as_bytes())
        .expect("Failed to write config content");

    // Process the files with parallel processing
    let options = ProcessingOptions {
        config_path: PathBuf::from(&config_path),
        dry_run: false,
    };

    let context = process_files(options).expect("Failed to process files");

    // Verify that all files were processed
    assert_eq!(context.stats.files_processed, 10);
    assert_eq!(context.stats.files_matched, 10);
    assert_eq!(context.stats.files_moved, 10);

    // Verify that the files were moved to the target directory
    let sorted_dir = target_dir.join("sorted");
    assert!(sorted_dir.exists());

    for i in 0..10 {
        let target_file = sorted_dir.join(format!("test_file_{i}.txt"));
        assert!(target_file.exists());
    }
}

#[test]
fn test_parallel_processing_dry_run() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let download_dir = temp_dir.path().join("download");
    let target_dir = temp_dir.path().join("target");

    // Create the download and target directories
    create_dir_all(&download_dir).expect("Failed to create download directory");
    create_dir_all(&target_dir).expect("Failed to create target directory");

    // Create test files in the download directory
    for i in 0..10 {
        let file_path = download_dir.join(format!("test_file_{i}.txt"));
        let mut file = File::create(&file_path).expect("Failed to create test file");
        writeln!(file, "Test content for file {i}").expect("Failed to write to test file");
    }

    // Create a test configuration file
    let config_path = temp_dir.path().join("../config.yaml");
    let config_content = format!(
        r#"
download:
  - {}
root:
  - - "{}"
rules:
  - title: Test Rule
    pattern: test_file_<num>.txt
    directory: "{}/sorted"
    "#,
        download_dir.to_string_lossy().replace('\\', "\\\\"),
        target_dir.to_string_lossy().replace('\\', "\\\\"),
        target_dir.to_string_lossy().replace('\\', "\\\\")
    );

    let mut config_file = File::create(&config_path).expect("Failed to create config file");
    config_file
        .write_all(config_content.as_bytes())
        .expect("Failed to write config content");

    // Process the files with parallel processing in dry-run mode
    let options = ProcessingOptions {
        config_path: PathBuf::from(&config_path),
        dry_run: true,
    };

    let context = process_files(options).expect("Failed to process files");

    // Verify that all files were processed
    assert_eq!(context.stats.files_processed, 10);
    assert_eq!(context.stats.files_matched, 10);
    assert_eq!(context.stats.files_moved, 0); // No files should be moved in dry-run mode

    // Verify that the planned operations were recorded
    assert_eq!(context.planned_operations.len(), 10);

    // Verify that the files were not moved to the target directory
    let sorted_dir = target_dir.join("sorted");
    assert!(!sorted_dir.exists());

    // Verify that all files are still in the download directory
    for i in 0..10 {
        let source_file = download_dir.join(format!("test_file_{i}.txt"));
        assert!(source_file.exists());
    }
}
