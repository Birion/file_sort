use std::env;

use file_sort::config::deserializer::expand_path;

#[test]
fn test_expand_path_with_env_variables() {
    // Set a test environment variable
    unsafe {
        env::set_var("TEST_ENV_VAR", "test_value");
    }
    
    // Test with environment variable
    let path = "$TEST_ENV_VAR/some/path";
    let expanded = expand_path(path);
    assert_eq!(expanded, "test_value/some/path");
    
    // Test with tilde and environment variable
    let path = "~/$TEST_ENV_VAR/some/path";
    let expanded = expand_path(path);
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE")).unwrap_or_default();
    let expected = format!("{}/test_value/some/path", home);
    assert_eq!(expanded, expected);
    
    // Clean up
    unsafe {
        env::remove_var("TEST_ENV_VAR");
    }
}

#[test]
fn test_expand_path_with_missing_env_variables() {
    // Make sure the environment variable doesn't exist
    unsafe {
        env::remove_var("NONEXISTENT_ENV_VAR");
    }
    
    // Test with non-existent environment variable
    let path = "$NONEXISTENT_ENV_VAR/some/path";
    let expanded = expand_path(path);
    
    // Should fall back to just expanding tilde (which does nothing in this case)
    assert_eq!(expanded, "$NONEXISTENT_ENV_VAR/some/path");
    
    // Test with tilde and non-existent environment variable
    let path = "~/$NONEXISTENT_ENV_VAR/some/path";
    let expanded = expand_path(path);
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE")).unwrap_or_default();
    let expected = format!("{}/$NONEXISTENT_ENV_VAR/some/path", home);
    assert_eq!(expanded, expected);
}

#[test]
fn test_expand_path_with_multiple_env_variables() {
    // Set test environment variables
    unsafe {
        env::set_var("TEST_ENV_VAR1", "test_value1");
        env::set_var("TEST_ENV_VAR2", "test_value2");
    }
    
    // Test with multiple environment variables
    let path = "$TEST_ENV_VAR1/$TEST_ENV_VAR2";
    let expanded = expand_path(path);
    assert_eq!(expanded, "test_value1/test_value2");
    
    // Clean up
    unsafe {
        env::remove_var("TEST_ENV_VAR1");
        env::remove_var("TEST_ENV_VAR2");
    }
}

#[test]
fn test_expand_path_with_windows_env_variables() {
    // Test with Windows-style environment variable syntax
    unsafe {
        env::set_var("TEST_WIN_VAR", "test_win_value");
    }
    
    // Test with Windows-style environment variable
    let path = "%TEST_WIN_VAR%\\some\\path";
    let expanded = expand_path(path);
    
    // shellexpand should handle both $VAR and %VAR% syntax
    assert_eq!(expanded, "test_win_value\\some\\path");
    
    // Clean up
    unsafe {
        env::remove_var("TEST_WIN_VAR");
    }
}

#[test]
fn test_expand_path_integration_with_config() {
    // This test would require more setup with actual Config objects
    // For now, we'll just test the basic functionality of expand_path
    
    // Set a test environment variable
    unsafe {
        env::set_var("TEST_CONFIG_VAR", "test_config_dir");
    }
    
    // Test with environment variable in a typical config path
    let path = "$TEST_CONFIG_VAR/downloads";
    let expanded = expand_path(path);
    assert_eq!(expanded, "test_config_dir/downloads");
    
    // Clean up
    unsafe {
        env::remove_var("TEST_CONFIG_VAR");
    }
}