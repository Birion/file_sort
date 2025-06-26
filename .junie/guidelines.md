# File Sort (fsort) Development Guidelines

This document provides guidelines and information for developers working on the File Sort (fsort) project.

## Build/Configuration Instructions

### Prerequisites

- Rust toolchain (rustc, cargo)
- Windows build tools (for Windows resource compilation)

### Building the Project

1. Clone the repository
2. Run `cargo build` to build the project
3. Run `cargo build --release` to build an optimised release version

### Windows-Specific Build Information

The project uses a custom build script (`build.rs`) to set the application icon for Windows builds. This requires the
`winres` crate as a build dependency.

## Testing Information

### Running Tests

- Run all tests: `cargo test`
- Run a specific test: `cargo test --test <test_name>`
- Run tests with output: `cargo test -- --nocapture`

### Adding New Tests

1. Create a new test file in the `tests` directory (e.g., `tests/my_feature_test.rs`)
2. Import the necessary modules from the crate
3. Write test functions with the `#[test]` attribute
4. Run the tests with `cargo test`

### Example Test

Here's a simple test that verifies the pattern handling in the Rule struct:

```rust
use file_sort::Rule;

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
    }
}
```

## Additional Development Information

### Project Structure

- `src/main.rs`: Entry point for the application
- `src/lib.rs`: Core library functionality
- `src/cli.rs`: Command-line interface handling
- `src/configuration.rs`: Configuration file parsing
- `src/parser.rs`: File pattern parsing
- `src/utils.rs`: Utility functions

### Code Style

- Follow the Rust standard style guidelines
- Use `cargo fmt` to format code
- Use `cargo clippy` for linting

### Error Handling

- The project uses `anyhow` for error handling
- Avoid using `unwrap()` or `expect()` in production code
- Propagate errors with the `?` operator

### Pattern Handling Notes

- The `clean_pattern` function removes all angle brackets from a pattern
- The `extract_pattern` function extracts content between angle brackets
- Be aware that nested angle brackets are not handled correctly by `extract_pattern`

### Configuration Files

- Configuration files use YAML format
- Rules can be defined with patterns, directories, and processors
- Transformative functions (First, Last) can be used to select specific directories