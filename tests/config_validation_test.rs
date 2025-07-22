use std::fs;
use tempfile::tempdir;

use file_sort::configuration::Config;

// Helper function to create a test configuration file and parse it
fn parse_test_config(config_content: &str) -> anyhow::Result<Config> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("test_config.yaml");

    fs::write(&config_path, config_content)?;

    // Use the special load_for_testing method that doesn't check path existence
    Config::load_for_testing(config_path)
}

#[test]
fn test_valid_configuration() {
    // A minimal valid configuration
    let valid_config = r#"
root:
  - - "~"
    - "Documents"
download:
  - "~"
  - "Downloads"
rules:
  - title: "Test Rule"
    pattern: "test.txt"
"#;

    let result = parse_test_config(valid_config);
    assert!(
        result.is_ok(),
        "Valid configuration failed validation: {:?}",
        result.err()
    );
}

#[test]
fn test_missing_root_directories() {
    let invalid_config = r#"
download:
  - "~"
  - "Downloads"
rules:
  - title: "Test Rule"
    pattern: "test.txt"
"#;

    let result = parse_test_config(invalid_config);
    assert!(
        result.is_err(),
        "Configuration with missing root directories should fail validation"
    );

    let error = result.err().unwrap();
    assert!(
        error.to_string().contains("missing field `root`"),
        "Error message should mention missing root field: {error}"
    );
}

#[test]
fn test_empty_root_directories() {
    let invalid_config = r#"
root: []
download:
  - "~"
  - "Downloads"
rules:
  - title: "Test Rule"
    pattern: "test.txt"
"#;

    let result = parse_test_config(invalid_config);
    assert!(
        result.is_err(),
        "Configuration with empty root directories should fail validation"
    );

    let error = result.err().unwrap();
    assert!(
        error.to_string().contains("No root directories specified"),
        "Error message should mention no root directories: {error}"
    );
}

#[test]
fn test_missing_download_directory() {
    let invalid_config = r#"
root:
  - - "~"
    - "Documents"
rules:
  - title: "Test Rule"
    pattern: "test.txt"
"#;

    let result = parse_test_config(invalid_config);
    assert!(
        result.is_err(),
        "Configuration with missing download directory should fail validation"
    );

    // The error will be from serde deserialization since download is a required field
    let error = result.err().unwrap();
    assert!(
        error.to_string().contains("missing field `download`"),
        "Error message should mention missing download directory: {error}"
    );
}

#[test]
fn test_missing_rules() {
    let invalid_config = r#"
root:
  - - "~"
    - "Documents"
download:
  - "~"
  - "Downloads"
rules: []
"#;

    let result = parse_test_config(invalid_config);
    assert!(
        result.is_err(),
        "Configuration with empty rules should fail validation"
    );

    let error = result.err().unwrap();
    assert!(
        error.to_string().contains("No rules specified"),
        "Error message should mention missing rules: {error}"
    );
}

#[test]
fn test_rule_without_title() {
    let invalid_config = r#"
root:
  - - "~"
    - "Documents"
download:
  - "~"
  - "Downloads"
rules:
  - title: ""
    pattern: "test.txt"
"#;

    let result = parse_test_config(invalid_config);
    assert!(
        result.is_err(),
        "Configuration with empty rule title should fail validation"
    );

    let error = result.err().unwrap();
    assert!(
        error.to_string().contains("empty title"),
        "Error message should mention empty title: {error}"
    );
}

#[test]
fn test_rule_without_pattern() {
    let invalid_config = r#"
root:
  - - "~"
    - "Documents"
download:
  - "~"
  - "Downloads"
rules:
  - title: "Test Rule"
"#;

    let result = parse_test_config(invalid_config);
    assert!(
        result.is_err(),
        "Configuration with missing pattern should fail validation"
    );

    let error = result.err().unwrap();
    assert!(
        error
            .to_string()
            .contains("no pattern or patterns specified"),
        "Error message should mention missing pattern: {error}"
    );
}

#[test]
fn test_rule_with_invalid_root_index() {
    let invalid_config = r#"
root:
  - - "~"
    - "Documents"
download:
  - "~"
  - "Downloads"
rules:
  - title: "Test Rule"
    pattern: "test.txt"
    root: 5
"#;

    let result = parse_test_config(invalid_config);
    assert!(
        result.is_err(),
        "Configuration with invalid root index should fail validation"
    );

    let error = result.err().unwrap();
    assert!(
        error.to_string().contains("out of bounds"),
        "Error message should mention out of bounds root index: {error}"
    );
}
