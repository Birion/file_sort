use std::fs;
use tempfile::tempdir;

use file_sort::config::{Config, Rule, ConfigProcessor, FormatConversion};
use file_sort::path_gen::FolderFunction;

#[test]
fn test_serialization_skips_empty_fields() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");

    // Create a rule with some fields set and others empty
    let rule = Rule {
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
    };

    // Create a configuration
    let config = Config {
        root: vec![temp_dir.path().to_path_buf()],
        download: temp_dir.path().to_path_buf(),
        rules: vec![rule],
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
    
    // Check that fields are included
    assert!(content.contains("Test Rule"));
    assert!(content.contains("test_pattern"));
    
    // Check that empty fields are excluded
    assert!(!content.contains("patterns:"));
    assert!(!content.contains("content_conditions:"));
    assert!(!content.contains("directory:"));
    assert!(!content.contains("function:"));
    assert!(!content.contains("processors:"));
}

#[test]
fn test_serialization_with_processors() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");

    // Create processors with some fields set and others empty
    let processors = ConfigProcessor {
        splitter: Some("-".to_string()),
        merger: None,
        pattern: Some("pattern".to_string()),
        date_format: None,
        replacement: Some("replacement".to_string()),
        format_conversion: None,
    };

    // Create a rule with processors
    let rule = Rule {
        title: "Test Rule".to_string(),
        pattern: Some("test_pattern".to_string()),
        patterns: None,
        content_conditions: None,
        match_all_conditions: true,
        directory: None,
        function: None,
        processors: Some(processors),
        root: 0,
        copy: false,
        old_pattern: String::new(),
        new_pattern: String::new(),
    };

    // Create a configuration
    let config = Config {
        root: vec![temp_dir.path().to_path_buf()],
        download: temp_dir.path().to_path_buf(),
        rules: vec![rule],
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
    
    // Check that fields are included
    assert!(content.contains("processors:"));
    assert!(content.contains("splitter: '-'"));
    assert!(content.contains("pattern: pattern"));
    assert!(content.contains("replacement: replacement"));
    
    // Check that empty fields are excluded
    assert!(!content.contains("merger:"));
    assert!(!content.contains("date_format:"));
    assert!(!content.contains("format_conversion:"));
}

#[test]
fn test_serialization_with_transformative_function() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");

    // Create a transformative function
    let function = FolderFunction::Last {
        args: Some(vec!["comics".to_string(), "batman".to_string()]),
    };

    // Create a rule with a transformative function
    let rule = Rule {
        title: "Test Rule".to_string(),
        pattern: Some("test_pattern".to_string()),
        patterns: None,
        content_conditions: None,
        match_all_conditions: true,
        directory: None,
        function: Some(function),
        processors: None,
        root: 0,
        copy: false,
        old_pattern: String::new(),
        new_pattern: String::new(),
    };

    // Create a configuration
    let config = Config {
        root: vec![temp_dir.path().to_path_buf()],
        download: temp_dir.path().to_path_buf(),
        rules: vec![rule],
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
    
    // Check that fields are included
    assert!(content.contains("function:"));
    assert!(content.contains("name: last"));
    assert!(content.contains("args:"));
    assert!(content.contains("- comics"));
    assert!(content.contains("- batman"));
}

#[test]
fn test_serialization_with_format_conversion() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");

    // Create format conversion
    let format_conversion = FormatConversion {
        source_format: "webp".to_string(),
        target_format: "jpg".to_string(),
        resize: Some((800, 600)),
    };

    // Create processors with format conversion
    let processors = ConfigProcessor {
        splitter: None,
        merger: None,
        pattern: None,
        date_format: None,
        replacement: None,
        format_conversion: Some(format_conversion),
    };

    // Create a rule with processors
    let rule = Rule {
        title: "Test Rule".to_string(),
        pattern: Some("test_pattern".to_string()),
        patterns: None,
        content_conditions: None,
        match_all_conditions: true,
        directory: None,
        function: None,
        processors: Some(processors),
        root: 0,
        copy: false,
        old_pattern: String::new(),
        new_pattern: String::new(),
    };

    // Create a configuration
    let config = Config {
        root: vec![temp_dir.path().to_path_buf()],
        download: temp_dir.path().to_path_buf(),
        rules: vec![rule],
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
    
    // Check that fields are included
    assert!(content.contains("processors:"));
    assert!(content.contains("format_conversion:"));
    assert!(content.contains("source_format: webp"));
    assert!(content.contains("target_format: jpg"));
    assert!(content.contains("resize:"));
    
    // Create another test with no resize
    let format_conversion_no_resize = FormatConversion {
        source_format: "webp".to_string(),
        target_format: "jpg".to_string(),
        resize: None,
    };

    let processors_no_resize = ConfigProcessor {
        splitter: None,
        merger: None,
        pattern: None,
        date_format: None,
        replacement: None,
        format_conversion: Some(format_conversion_no_resize),
    };

    let rule_no_resize = Rule {
        title: "Test Rule No Resize".to_string(),
        pattern: Some("test_pattern".to_string()),
        patterns: None,
        content_conditions: None,
        match_all_conditions: true,
        directory: None,
        function: None,
        processors: Some(processors_no_resize),
        root: 0,
        copy: false,
        old_pattern: String::new(),
        new_pattern: String::new(),
    };

    let config_no_resize = Config {
        root: vec![temp_dir.path().to_path_buf()],
        download: temp_dir.path().to_path_buf(),
        rules: vec![rule_no_resize],
        files: vec![],
        parent: None,
    };

    let yaml_no_resize = serde_yaml::to_string(&config_no_resize).unwrap();
    let config_path_no_resize = temp_dir.path().join("test_config_no_resize.yaml");
    fs::write(&config_path_no_resize, yaml_no_resize).unwrap();

    let content_no_resize = fs::read_to_string(&config_path_no_resize).unwrap();
    
    // Check that resize field is excluded
    assert!(content_no_resize.contains("format_conversion:"));
    assert!(content_no_resize.contains("source_format: webp"));
    assert!(content_no_resize.contains("target_format: jpg"));
    assert!(!content_no_resize.contains("resize:"));
}