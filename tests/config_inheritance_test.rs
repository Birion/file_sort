use std::path::PathBuf;
use file_sort::config::loader::load_config_for_testing;

#[test]
fn test_config_inheritance() {
    // Load the child configuration file
    let config_path = PathBuf::from("tests/configs/child_config.yaml");
    let config = load_config_for_testing(config_path).unwrap();
    
    // Verify that the child configuration has inherited the root directories from the parent
    assert_eq!(config.root.len(), 2);
    
    // Verify that the rules from both parent and child are present
    // The child config has 2 rules, and the parent has 3 rules, so we should have 5 rules total
    assert_eq!(config.rules.len(), 5);
    
    // Verify specific rules from parent and child are present
    let rule_titles: Vec<String> = config.rules.iter().map(|r| r.title.clone()).collect();
    
    // Parent rules
    assert!(rule_titles.contains(&"Darths & Droids".to_string()));
    assert!(rule_titles.contains(&"Gunnerkrig Court".to_string()));
    assert!(rule_titles.contains(&"Girl Genius".to_string()));
    
    // Child rules
    assert!(rule_titles.contains(&"El Goonish Shive".to_string()));
    assert!(rule_titles.contains(&"Sequential Arts".to_string()));
}