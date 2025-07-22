use std::path::PathBuf;

use serde::Deserialize;

use crate::errors::Result;
use crate::transformative_function::TransformativeFunction;
use crate::utils::{clean_pattern, extract_pattern};

/// Configuration for processing file paths
///
/// Defines how file paths should be processed, including date formatting
/// and pattern replacement.
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
    pub function: Option<TransformativeFunction>,
    /// Optional processors to apply to the file path
    pub processors: Option<ConfigProcessor>,
    /// The index of the root directory to use
    #[serde(default)]
    pub root: usize,
    /// Whether to copy the file instead of moving it
    #[serde(default)]
    pub copy: bool,
    /// The processed pattern without angle brackets
    #[serde(skip_deserializing)]
    pub old_pattern: String,
    /// The extracted pattern from between angle brackets
    #[serde(skip_deserializing)]
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
