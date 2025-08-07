# File Sort (fsort) Improvement Tasks

This document contains a comprehensive list of actionable improvement tasks for the File Sort utility. Each task is
logically ordered and covers both architectural and code-level improvements.

## Code Quality and Structure

1. [x] Refactor error handling to use custom error types instead of anyhow::Error for more specific error messages
2. [x] Implement a proper logging system with different verbosity levels instead of direct println! calls
3. [x] Add comprehensive documentation comments to all public functions and types
4. [x] Extract hardcoded constants to a dedicated configuration module
5. [x] Improve code organisation by splitting large files into smaller, more focused modules
6. [x] Implement proper error propagation instead of using unwrap() and expect()
7. [x] Add input validation for configuration files with helpful error messages
8. [x] Refactor the Processor struct to follow the builder pattern for better readability
9. [x] Implement proper unit tests for all core functionality

## Feature Enhancements

1. [x] Add support for converting file formats (e.g. image resizing, text encoding conversion)
2. [x] Add support for multiple configuration files with inheritance
3. [x] Implement a dry-run mode with detailed output of planned operations
4. [x] Add support for file-content-based rules using file metadata or content analysis
5. [x] Implement parallel processing for file operations to improve performance
6. [x] Add support for conditional rules based on file properties (size, date, content type)
7. [ ] Implement a configuration wizard for creating new config files
8. [ ] Add support for environment variable expansion in configuration paths
9. [ ] Implement a plugin system for custom processors
10. [ ] Add support for different configuration formats (TOML, INI) in addition to YAML
11. [ ] Implement incremental processing (only process new or changed files)

## User Experience

1. [ ] Add progress indicators for operations on large numbers of files
2. [ ] Implement colorized output for better visual distinction (expand current implementation)
3. [ ] Add an interactive mode for confirming file operations
4. [ ] Improve error messages with suggestions for resolution
5. [ ] Add a verbose mode for detailed logging
6. [ ] Implement a configuration validation command with helpful error messages
7. [ ] Add a command to list all available rules in the configuration
8. [ ] Add support for undo operations

## Testing and Reliability

1. [ ] Expand unit test coverage for core components
2. [ ] Add integration tests for end-to-end workflows
3. [ ] Implement property-based testing for rule processing
4. [ ] Add stress testing for large file collections
5. [ ] Implement automated regression testing
6. [ ] Add benchmarking tools for performance testing
7. [ ] Implement error recovery mechanisms for failed operations
8. [ ] Add transaction-like operations to ensure atomicity of file movements

## Documentation

1. [ ] Create comprehensive user documentation with examples
2. [ ] Add inline code documentation for better maintainability
3. [ ] Create a quick-start guide for new users
4. [ ] Document common patterns and use cases
5. [ ] Add a troubleshooting guide for common issues
6. [ ] Create a changelog to track version changes
7. [ ] Document the configuration file format with examples
8. [ ] Add architecture documentation explaining the design decisions

## Performance Optimisation

1. [ ] Optimize regex compilation and reuse
2. [ ] Add caching for frequently used patterns and transformations
3. [ ] Implement lazy loading of configuration files
4. [ ] Optimise file system operations by batching similar operations
5. [ ] Reduce memory usage for large file collections
6. [ ] Implement more efficient pattern matching algorithms

## Cross-Platform Compatibility

1. [ ] Ensure proper path handling on different operating systems
2. [ ] Add platform-specific optimisations for file operations
3. [ ] Implement proper Unicode support for filenames
4. [ ] Ensure configuration paths work correctly on all supported platforms
5. [ ] Add support for platform-specific file attributes

## Security

1. [ ] Implement proper permission checking before file operations
2. [ ] Add safeguards against moving system or critical files
3. [ ] Implement configuration file validation to prevent malicious configurations
4. [ ] Add support for secure credential storage for remote file operations
5. [ ] Implement proper handling of symbolic links and hard links
