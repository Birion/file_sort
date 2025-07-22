use file_sort::rules::{Rule, ConfigProcessor};
use file_sort::transformative_function::TransformativeFunction;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_make_patterns() {
        // Test with angle brackets
        let mut rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pattern>".to_string()),
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
        assert_eq!(rule.old_pattern, "pattern");
        assert_eq!(rule.new_pattern, "pattern");

        // Test with multiple angle brackets
        let mut rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pat<ter>n>".to_string()),
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
        assert_eq!(rule.old_pattern, "pattern");
        assert_eq!(rule.new_pattern, "ter>n");

        // Test with no angle brackets
        let mut rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("pattern".to_string()),
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
        assert_eq!(rule.old_pattern, "pattern");
        assert_eq!(rule.new_pattern, "pattern");

        // Test with empty pattern
        let mut rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("".to_string()),
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
        assert_eq!(rule.old_pattern, "");
        assert_eq!(rule.new_pattern, "");
    }

    #[test]
    fn test_rule_with_directory() {
        // Test with directory as string
        let rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: Some(PathBuf::from("test_dir")),
            function: None,
            processors: None,
            root: 0,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };

        assert_eq!(rule.directory, Some(PathBuf::from("test_dir")));

        // Test with directory as path
        let rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: Some(PathBuf::from("nested/test_dir")),
            function: None,
            processors: None,
            root: 0,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };

        assert_eq!(rule.directory, Some(PathBuf::from("nested/test_dir")));
    }

    #[test]
    fn test_rule_with_transformative_function() {
        // Test with First function
        let rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: None,
            function: Some(TransformativeFunction::First { args: None }),
            processors: None,
            root: 0,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };

        assert_eq!(rule.function, Some(TransformativeFunction::First { args: None }));

        // Test with Last function
        let rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: None,
            function: Some(TransformativeFunction::Last { args: None }),
            processors: None,
            root: 0,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };

        assert_eq!(rule.function, Some(TransformativeFunction::Last { args: None }));
    }

    #[test]
    fn test_rule_with_processors() {
        // Test with all processor fields
        let rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: None,
            function: None,
            processors: Some(ConfigProcessor {
                splitter: Some("_".to_string()),
                merger: Some("-".to_string()),
                pattern: Some("test".to_string()),
                date_format: Some("%Y-%m-%d".to_string()),
                replacement: Some("replaced".to_string()),
            }),
            root: 0,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };

        let processors = rule.processors.unwrap();
        assert_eq!(processors.splitter, Some("_".to_string()));
        assert_eq!(processors.merger, Some("-".to_string()));
        assert_eq!(processors.pattern, Some("test".to_string()));
        assert_eq!(processors.date_format, Some("%Y-%m-%d".to_string()));
        assert_eq!(processors.replacement, Some("replaced".to_string()));

        // Test with explicit merger
        let rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: None,
            function: None,
            processors: Some(ConfigProcessor {
                splitter: Some("_".to_string()),
                merger: None,
                pattern: None,
                date_format: None,
                replacement: None,
            }),
            root: 0,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };

        let processors = rule.processors.unwrap();
        assert_eq!(processors.merger, None); // When creating directly, merger is None
    }

    #[test]
    fn test_rule_with_copy_flag() {
        // Test with copy flag set to true
        let rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: None,
            function: None,
            processors: None,
            root: 0,
            copy: true,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };

        assert!(rule.copy);

        // Test with copy flag set to false
        let rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: None,
            function: None,
            processors: None,
            root: 0,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };

        assert!(!rule.copy);
    }

    #[test]
    fn test_rule_with_root_index() {
        // Test with root index set to 0
        let rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: None,
            function: None,
            processors: None,
            root: 0,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };

        assert_eq!(rule.root, 0);

        // Test with root index set to 1
        let rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: None,
            function: None,
            processors: None,
            root: 1,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };

        assert_eq!(rule.root, 1);
    }
}