/// Constants used throughout the application
///
/// This module centralises all constants used in the application to make
/// them easier to manage and update.

/// Wildcard character used in path patterns
///
/// This constant is used when constructing paths with glob patterns.
pub const WILDCARD: &str = "*";

/// Qualifier string used for application identification
///
/// This is used as part of the application's unique identifier.
pub const QUALIFIER: &str = "com";

/// Organisation name used for application identification
///
/// This is used as part of the application's unique identifier.
pub const ORGANIZATION: &str = "Ondřej Vágner";

/// Application name used for identification
///
/// This is the name of the application used in various contexts like
/// configuration file paths and application identification.
pub const APPLICATION: &str = "comic_sort";

/// Help text for the config command-line option
pub const CONFIG_HELP: &str = "Read from a specific config file";

/// Help text for the dry-run command-line option
pub const DRY_RUN_HELP: &str = "Run without moving any files";

/// Help text for the verbose command-line option
pub const VERBOSE_HELP: &str = "Increase verbosity level (can be used multiple times)";

/// Default path for the configuration file
pub const DEFAULT_CONFIG_PATH: &str = "config.yaml";
