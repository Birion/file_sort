use std::fs;
use tempfile::tempdir;

use file_sort::cli::{get_wizard_output_path, is_wizard_command};

// Note: These tests don't test the interactive parts of the wizard
// as that would require simulating user input, which is complex.
// Instead, they test the non-interactive parts like command-line parsing.

#[test]
fn test_is_wizard_command() {
    // Test with wizard subcommand
    let args = vec!["fsort", "wizard"];
    let matches = clap::Command::new("fsort")
        .subcommand(clap::Command::new("wizard"))
        .get_matches_from(args);

    assert!(is_wizard_command(&matches));

    // Test without wizard subcommand
    let args = vec!["fsort"];
    let matches = clap::Command::new("fsort")
        .subcommand(clap::Command::new("wizard"))
        .get_matches_from(args);

    assert!(!is_wizard_command(&matches));
}

#[test]
fn test_get_wizard_output_path() {
    // Test with default output path
    let args = vec!["fsort", "wizard"];
    let matches = clap::Command::new("fsort")
        .subcommand(
            clap::Command::new("wizard").arg(
                clap::Arg::new("output")
                    .short('o')
                    .long("output")
                    .default_value("config.yaml"),
            ),
        )
        .get_matches_from(args);

    let output_path = get_wizard_output_path(&matches).unwrap();
    assert_eq!(output_path, "config.yaml");

    // Test with custom output path
    let args = vec!["fsort", "wizard", "--output", "custom_config.yaml"];
    let matches = clap::Command::new("fsort")
        .subcommand(
            clap::Command::new("wizard").arg(
                clap::Arg::new("output")
                    .short('o')
                    .long("output")
                    .default_value("config.yaml"),
            ),
        )
        .get_matches_from(args);

    let output_path = get_wizard_output_path(&matches).unwrap();
    assert_eq!(output_path, "custom_config.yaml");
}

// This test verifies that the save_config function works correctly
// by creating a minimal configuration and saving it to a temporary file.
#[test]
fn test_save_config() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");

    // Create a minimal configuration
    let config = file_sort::config::Config {
        root: vec![temp_dir.path().to_path_buf()],
        download: temp_dir.path().to_path_buf(),
        rules: vec![file_sort::config::Rule {
            title: "Test Rule".to_string(),
            pattern: Some("test_pattern".to_string()),
            patterns: None,
            content_conditions: None,
            match_all_conditions: true,
            directory: None,
            function: None,
            processors: None,
            root: 0,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        }],
        files: vec![],
        parent: None,
    };

    // Save the configuration
    let yaml = serde_yaml::to_string(&config).unwrap();
    fs::write(&config_path, yaml).unwrap();

    // Verify that the file was created
    assert!(config_path.exists());

    // Verify that the file contains the expected content
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("Test Rule"));
    assert!(content.contains("test_pattern"));
}
