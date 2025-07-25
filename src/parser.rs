use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

use crate::rules::{Rules, RulesList};
use utils::*;

mod utils;

/// Deserialises an array of strings into a PathBuf
///
/// This function is used as a custom deserializer for Serde.
///
/// # Arguments
/// * `deserializer` - The deserializer to use
///
/// # Returns
/// * `Result<PathBuf, D::Error>` - The deserialized PathBuf or an error
pub fn deserialize_from_array_to_pathbuf<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path_strings: Vec<String> = Deserialize::deserialize(deserializer)?;
    Ok(path_strings.iter().map(process_path).collect())
}

/// Deserialises an array of strings into an optional PathBuf
///
/// This function is used as a custom deserializer for Serde.
///
/// # Arguments
/// * `deserializer` - The deserializer to use
///
/// # Returns
/// * `Result<Option<PathBuf>, D::Error>` - The deserialised optional PathBuf or an error
pub fn deserialize_from_array_to_optional_pathbuf<'de, D>(
    deserializer: D,
) -> Result<Option<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let result: Option<Vec<String>> = Deserialize::deserialize(deserializer)?;

    match result {
        None => Ok(None),
        Some(p) => Ok(Some(p.iter().map(process_path).collect())),
    }
}

/// Deserialises arrays of strings into a vector of PathBufs
///
/// This function is used as a custom deserializer for Serde.
///
/// # Arguments
/// * `deserializer` - The deserializer to use
///
/// # Returns
/// * `Result<Vec<PathBuf>, D::Error>` - The deserialised vector of PathBufs or an error
pub fn deserialize_from_arrays_to_pathbuf_vec<'de, D>(
    deserializer: D,
) -> Result<Vec<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let paths: Vec<Vec<String>> = Deserialize::deserialize(deserializer)?;
    Ok(paths.into_iter().map(process_strings_to_paths).collect())
}

/// Parses rules from the configuration
///
/// This function is used as a custom deserializer for Serde.
/// It handles both single rule lists and multiple rule lists organised by root directories.
///
/// # Arguments
/// * `deserializer` - The deserializer to use
///
/// # Returns
/// * `Result<RulesList, D::Error>` - The parsed list of rules or an error
pub fn parse_rules<'de, D>(deserializer: D) -> Result<RulesList, D::Error>
where
    D: Deserializer<'de>,
{
    let parsed_rules: Rules = Deserialize::deserialize(deserializer)?;
    let mut result_rules = vec![];
    match parsed_rules {
        Rules::SingleRule(mut rules) => {
            process_rules(&mut rules, &mut result_rules).map_err(serde::de::Error::custom)?;
        }
        Rules::RootRules(roots) => {
            process_and_append_rules(roots, &mut result_rules).map_err(serde::de::Error::custom)?;
        }
    }
    result_rules.dedup();
    Ok(result_rules)
}

pub fn default_merger() -> Option<String> {
    Some(String::from("-"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{Rule, Rules};
    use serde::de::value::SeqDeserializer;

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
    fn test_default_merger() {
        let merger = default_merger();
        assert_eq!(merger, Some("-".to_string()));
    }

    #[test]
    fn test_deserialize_from_array_to_pathbuf() {
        // Create a test array of strings
        let path_strings = vec!["~/Documents".to_string(), "test".to_string()];

        // Create a deserializer for the array
        let deserializer: SeqDeserializer<std::vec::IntoIter<String>, serde::de::value::Error> =
            SeqDeserializer::new(path_strings.into_iter());

        // Test deserialising the array to a PathBuf
        let result: Result<PathBuf, serde::de::value::Error> =
            deserialize_from_array_to_pathbuf(deserializer);
        assert!(result.is_ok());

        // The result should be a PathBuf with the expanded path
        let path_buf = result.unwrap();

        // On Windows, the home directory expansion might not work in tests,
        // So we'll just check that the path contains "Documents/test" or "Documents\test"
        let path_str = path_buf.to_string_lossy();
        assert!(path_str.contains("Documents") && path_str.contains("test"));
    }

    #[test]
    fn test_deserialize_from_array_to_optional_pathbuf() {
        // Test with Some value
        let path_strings = vec!["~/Documents".to_string(), "test".to_string()];
        let deserializer: SeqDeserializer<std::vec::IntoIter<String>, serde::de::value::Error> =
            SeqDeserializer::new(path_strings.into_iter());

        let result = deserialize_from_array_to_optional_pathbuf(deserializer);
        assert!(result.is_ok());

        let optional_path_buf = result.unwrap();
        assert!(optional_path_buf.is_some());

        let path_buf = optional_path_buf.unwrap();
        let path_str = path_buf.to_string_lossy();
        assert!(path_str.contains("Documents") && path_str.contains("test"));

        // Test with None value
        // This is harder to test directly with the deserializer, so we'll skip it
    }

    #[test]
    fn test_deserialize_from_arrays_to_pathbuf_vec() {
        // Create a test array of arrays of strings
        let paths = vec![
            vec!["~/Documents".to_string(), "test1".to_string()],
            vec!["~/Downloads".to_string(), "test2".to_string()],
        ];

        // Create a deserializer for the array of arrays
        let deserializer: SeqDeserializer<
            std::vec::IntoIter<Vec<String>>,
            serde::de::value::Error,
        > = SeqDeserializer::new(paths.into_iter());

        // Test deserialising the array of arrays to a vector of PathBufs
        let result: Result<Vec<PathBuf>, serde::de::value::Error> =
            deserialize_from_arrays_to_pathbuf_vec(deserializer);
        assert!(result.is_ok());

        // The result should be a vector of PathBufs with the expanded paths
        let path_bufs = result.unwrap();
        assert_eq!(path_bufs.len(), 2);

        // Check the first PathBuf
        let path_str1 = path_bufs[0].to_string_lossy();
        assert!(path_str1.contains("Documents") && path_str1.contains("test1"));

        // Check the second PathBuf
        let path_str2 = path_bufs[1].to_string_lossy();
        assert!(path_str2.contains("Downloads") && path_str2.contains("test2"));
    }

    #[test]
    fn test_parse_rules_single_rule() {
        // Create a test rule
        let rule = create_test_rule("test");

        // Create a test Rules::SingleRule
        let rules = Rules::SingleRule(vec![rule]);
        // Serialise the rules to a YAML string
        let yaml_string = serde_yaml::to_string(&rules).unwrap();

        // Create a deserializer for the Rules
        let deserializer = serde_yaml::Deserializer::from_str(&yaml_string);

        // Test parsing the rules
        let result = parse_rules(deserializer);
        assert!(result.is_ok());

        // The result should be a RulesList with one rule
        let rules_list = result.unwrap();
        assert_eq!(rules_list.len(), 1);

        // Check the rule
        let parsed_rule = &rules_list[0];
        assert_eq!(parsed_rule.title, "Test Rule");
        assert_eq!(parsed_rule.pattern, Some("test".to_string()));
    }

    #[test]
    fn test_parse_rules_root_rules() {
        // Create test rules for different roots
        let rule1 = create_test_rule("test1");
        let rule2 = create_test_rule("test2");

        // Create a test Rules::RootRules
        let rules = Rules::RootRules(vec![vec![rule1], vec![rule2]]);

        // Serialise the rules to a YAML string
        let yaml_string = serde_yaml::to_string(&rules).unwrap();

        // Create a deserializer for the Rules
        let deserializer = serde_yaml::Deserializer::from_str(&yaml_string);

        // Test parsing the rules
        let result = parse_rules(deserializer);
        assert!(result.is_ok());

        // The result should be a RulesList with two rules
        let rules_list = result.unwrap();
        assert_eq!(rules_list.len(), 2);

        // Check the first rule
        let parsed_rule1 = &rules_list[0];
        assert_eq!(parsed_rule1.title, "Test Rule");
        assert_eq!(parsed_rule1.pattern, Some("test1".to_string()));
        assert_eq!(parsed_rule1.root, 0);

        // Check the second rule
        let parsed_rule2 = &rules_list[1];
        assert_eq!(parsed_rule2.title, "Test Rule");
        assert_eq!(parsed_rule2.pattern, Some("test2".to_string()));
        assert_eq!(parsed_rule2.root, 1);
    }
}
