use file_sort::processor::ProcessorBuilder;
use file_sort::rules::Rule;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test rule
    fn create_test_rule(pattern: &str) -> Rule {
        let mut rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some(pattern.to_string()),
            patterns: None,
            directory: None,
            function: None,
            processors: None,
            root: 0,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };
        rule.make_patterns().unwrap();
        rule
    }

    #[test]
    fn test_processor_builder() {
        // Create a test processor with a filename
        let processor = ProcessorBuilder::new(Path::new("test_123_filename.txt"))
            .target(PathBuf::from("target_dir"))
            .build();

        // Verify that the source path is set correctly
        assert_eq!(processor.source(), &PathBuf::from("test_123_filename.txt"));

        // Verify that the target path is set correctly
        assert_eq!(processor.target(), &PathBuf::from("target_dir"));
    }
}
