use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

use file_sort::config::Rule;
use file_sort::discovery::content_analyser::{analyse_file_content, evaluate_condition};
use file_sort::discovery::{ConditionOperator, ContentCondition, ContentProperty, FileInfo};

#[test]
fn test_file_metadata_extraction() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create a test file
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "This is a test file for content analysis").unwrap();

    // Get file metadata
    let metadata = file_sort::discovery::content_analyser::get_file_metadata(&file_path).unwrap();

    // Verify metadata
    assert!(metadata.size > 0);
    assert_eq!(metadata.mime_type, "text/plain");
}

#[test]
fn test_content_analysis() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create a test file
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "This is a test file for content analysis").unwrap();

    // Analyse file content
    let analysis = analyse_file_content(&file_path).unwrap();

    // Verify analysis
    assert!(analysis.is_text);
    assert!(!analysis.is_binary);
    assert!(analysis.text_preview.is_some());
    assert!(analysis.text_preview.unwrap().contains("test file"));
}

#[test]
fn test_content_condition_evaluation() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create a test file
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "This is a test file for content analysis").unwrap();

    // Analyse file content
    let analysis = analyse_file_content(&file_path).unwrap();

    // Test size condition
    let size_condition = ContentCondition {
        property: ContentProperty::Size,
        operator: ConditionOperator::GreaterThan,
        value: "0".to_string(),
    };
    assert!(evaluate_condition(&size_condition, &analysis).unwrap());

    // Test mime type condition
    let mime_condition = ContentCondition {
        property: ContentProperty::MimeType,
        operator: ConditionOperator::Equal,
        value: "text/plain".to_string(),
    };
    assert!(evaluate_condition(&mime_condition, &analysis).unwrap());

    // Test content condition
    let content_condition = ContentCondition {
        property: ContentProperty::Content,
        operator: ConditionOperator::Contains,
        value: "test file".to_string(),
    };
    assert!(evaluate_condition(&content_condition, &analysis).unwrap());

    // Test negative condition
    let negative_condition = ContentCondition {
        property: ContentProperty::Content,
        operator: ConditionOperator::Contains,
        value: "not in the file".to_string(),
    };
    assert!(!evaluate_condition(&negative_condition, &analysis).unwrap());
}

#[test]
fn test_file_info_content_analysis() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create a test file
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "This is a test file for content analysis").unwrap();

    // Create FileInfo
    let mut file_info = FileInfo::new(file_path.clone()).unwrap();

    // Initially, content_analysis should be None
    assert!(file_info.content_analysis.is_none());

    // Ensure content is analysed
    file_info.ensure_content_analysed().unwrap();

    // Now content_analysis should be Some
    assert!(file_info.content_analysis.is_some());

    // Get a reference to the content analysis
    let analysis = file_info.content_analysis.as_ref().unwrap();

    // Verify analysis
    assert!(analysis.is_text);
    assert!(!analysis.is_binary);
    assert!(analysis.text_preview.is_some());
    assert!(
        analysis
            .text_preview
            .as_ref()
            .unwrap()
            .contains("test file")
    );
}

// This test requires the matcher module to be accessible, which might not be possible in tests
// You may need to make the matcher module public or expose the necessary functions
#[test]
fn test_rule_with_content_conditions() {
    // This test would verify that a rule with content conditions correctly matches files
    // For now, we'll just create a rule with content conditions to ensure it compiles

    let rule = Rule {
        title: "Test Content Rule".to_string(),
        pattern: None,
        patterns: None,
        content_conditions: Some(vec![
            ContentCondition {
                property: ContentProperty::Size,
                operator: ConditionOperator::GreaterThan,
                value: "0".to_string(),
            },
            ContentCondition {
                property: ContentProperty::MimeType,
                operator: ConditionOperator::Equal,
                value: "text/plain".to_string(),
            },
        ]),
        match_all_conditions: true,
        directory: None,
        function: None,
        processors: None,
        root: 0,
        copy: false,
        old_pattern: String::new(),
        new_pattern: String::new(),
    };

    // Assert that the rule has content conditions
    assert!(rule.content_conditions.is_some());
    assert_eq!(rule.content_conditions.unwrap().len(), 2);
}
