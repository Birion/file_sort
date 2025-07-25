use std::path::PathBuf;

use serde::Deserialize;

use crate::errors::Result;
use crate::folder_function::FolderFunction;
use crate::utils::{clean_pattern, extract_pattern};

/// Represents a file format conversion configuration
///
/// Specifies the source and target formats for file conversion
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct FormatConversion {
    /// Source file format (e.g., "jpg", "png", "utf-8")
    #[serde(alias = "from")]
    pub source_format: String,
    /// Target file format (e.g., "png", "webp", "utf-16")
    #[serde(alias = "to")]
    pub target_format: String,
    /// Optional resize dimensions for image conversion (width, height)
    pub resize: Option<(u32, u32)>,
}

/// Configuration for processing file paths
///
/// Defines how file paths should be processed, including date formatting,
/// pattern replacement, and file format conversion.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ConfigProcessor {
    /// String to split the filename with
    pub splitter: Option<String>,
    /// String to merge parts of the filename with
    #[serde(default = "default_merger")]
    pub merger: Option<String>,
    /// Regex pattern to match in the filename
    pub pattern: Option<String>,
    /// Format string for date processing
    pub date_format: Option<String>,
    /// Replacement string for pattern matching
    pub replacement: Option<String>,
    /// File format conversion configuration
    pub format_conversion: Option<FormatConversion>,
}

/// Default merger function for ConfigProcessor
fn default_merger() -> Option<String> {
    Some(" ".to_string())
}

/// A list of rules for file sorting.
///
/// This type is used throughout the application to represent collections of sorting rules.
pub type RulesList = Vec<Rule>;

/// Represents different types of rule configurations
///
/// This enum allows for both single rule lists and multiple rule lists
/// organised by root directories.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Rules {
    /// A single list of rules
    SingleRule(RulesList),
    /// Multiple lists of rules organised by root directories
    RootRules(Vec<RulesList>),
}

/// Represents a rule for file sorting
///
/// A rule defines how files should be matched and where they should be moved or copied.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Rule {
    /// The title of the rule
    pub title: String,
    /// The pattern to match files against
    pub pattern: Option<String>,
    /// Multiple patterns to match files against
    pub patterns: Option<Vec<String>>,
    /// The directory to move or copy files to
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_from_array_to_optional_pathbuf")]
    pub directory: Option<PathBuf>,
    /// Optional transformative function to apply to the directory
    pub function: Option<FolderFunction>,
    /// Optional processors to apply to the file path
    pub processors: Option<ConfigProcessor>,
    /// The index of the root directory to use
    #[serde(default)]
    pub root: usize,
    /// Whether to copy the file instead of moving it
    #[serde(default)]
    pub copy: bool,
    /// The processed pattern without angle brackets
    #[serde(skip_deserializing, skip_serializing)]
    pub old_pattern: String,
    /// The extracted pattern from between angle brackets
    #[serde(skip_deserializing, skip_serializing)]
    pub new_pattern: String,
}

impl Rule {
    /// Processes the rule's pattern to extract the old and new patterns
    ///
    /// This method extracts patterns from the rule's pattern string. It sets:
    /// - `old_pattern`: The pattern with angle brackets removed
    /// - `new_pattern`: The content between angle brackets
    ///
    /// # Returns
    /// * `Result<()>` - Ok if pattern processing succeeds, or an error
    ///
    /// # Errors
    /// * Returns an error if pattern cleaning or extraction fails
    ///
    /// # Examples
    /// ```
    /// # use file_sort::rules::Rule;
    /// # use file_sort::errors::Error;
    /// let mut rule = Rule {
    ///     title: "Test Rule".to_string(),
    ///     pattern: Some("<pattern>".to_string()),
    ///     patterns: None,
    ///     directory: None,
    ///     function: None,
    ///     processors: None,
    ///     root: 0,
    ///     copy: false,
    ///     old_pattern: String::new(),
    ///     new_pattern: String::new(),
    /// };
    ///
    /// rule.make_patterns()?;
    /// assert_eq!(rule.old_pattern, "pattern");
    /// assert_eq!(rule.new_pattern, "pattern");
    /// # Ok::<(), Error>(())
    /// ```
    pub fn make_patterns(&mut self) -> Result<()> {
        if let Some(pattern) = &self.pattern {
            self.old_pattern = clean_pattern(pattern.as_str())?;
            self.new_pattern = extract_pattern(pattern.as_str())?;
        }
        Ok(())
    }
}

/// Deserialises a value from an array to an optional PathBuf
///
/// This function is used to deserialise a directory field in a Rule struct.
/// It can handle both string values and arrays of strings.
fn deserialize_from_array_to_optional_pathbuf<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<PathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct OptionalPathBufVisitor;

    impl<'de> serde::de::Visitor<'de> for OptionalPathBufVisitor {
        type Value = Option<PathBuf>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or array of strings")
        }

        fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(PathBuf::from(value)))
        }

        fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(self)
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut path = PathBuf::new();
            while let Some(segment) = seq.next_element::<String>()? {
                path.push(segment);
            }
            Ok(Some(path))
        }
    }

    deserializer.deserialize_any(OptionalPathBufVisitor)
}

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
        assert_eq!(rule.new_pattern, "ter");

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
            function: Some(FolderFunction::First { args: None }),
            processors: None,
            root: 0,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };

        assert_eq!(rule.function, Some(FolderFunction::First { args: None }));

        // Test with Last function
        let rule = Rule {
            title: "Test Rule".to_string(),
            pattern: Some("<pattern>".to_string()),
            patterns: None,
            directory: None,
            function: Some(FolderFunction::Last { args: None }),
            processors: None,
            root: 0,
            copy: false,
            old_pattern: String::new(),
            new_pattern: String::new(),
        };

        assert_eq!(rule.function, Some(FolderFunction::Last { args: None }));
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
                format_conversion: None,
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
                format_conversion: None,
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
