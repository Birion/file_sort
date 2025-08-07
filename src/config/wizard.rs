//! Configuration wizard functionality
//!
//! This module contains functions for creating a new configuration file with a wizard.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use log::debug;

use crate::config::model::{Config, Rule};
use crate::discovery::{ConditionOperator, ContentCondition, ContentProperty};

/// Creates a new configuration file with a wizard
///
/// This function guides the user through creating a new configuration file
/// by asking questions about root directories, download directory, and rules.
///
/// # Arguments
/// * `output_path` - Path where the new configuration file will be saved
///
/// # Returns
/// * `Result<()>` - Success or an error
///
/// # Errors
/// Returns an error if the configuration file cannot be created
pub fn create_config_with_wizard(output_path: &str) -> Result<()> {
    println!("Welcome to the File Sort Configuration Wizard!");
    println!("This wizard will help you create a new configuration file.");
    println!("Press Ctrl+C at any time to cancel.");
    println!();

    // Create a new configuration
    let mut config = Config {
        root: Vec::new(),
        download: PathBuf::new(),
        rules: Vec::new(),
        files: Vec::new(),
        parent: None,
    };

    // Ask for root directories
    println!("First, let's set up your root directories.");
    println!("These are the directories where files will be moved or copied to.");
    println!("You can add multiple root directories.");
    println!();

    loop {
        println!("Enter a root directory path (or leave empty to finish adding roots):");
        let root = read_line()?;
        if root.trim().is_empty() {
            if config.root.is_empty() {
                println!("You must add at least one root directory.");
                continue;
            }
            break;
        }

        let root_path = PathBuf::from(root);
        if !root_path.exists() {
            println!("Warning: Directory does not exist: {}", root_path.display());
            println!("Do you want to create it? (y/n)");
            let create = read_line()?.trim().to_lowercase();
            if create == "y" || create == "yes" {
                fs::create_dir_all(&root_path)?;
                println!("Directory created: {}", root_path.display());
            } else {
                println!("Directory not created. Please enter a different path.");
                continue;
            }
        }

        if !root_path.is_dir() {
            println!("Error: Path is not a directory: {}", root_path.display());
            println!("Please enter a directory path.");
            continue;
        }

        config.root.push(root_path);
        println!(
            "Root directory added: {}",
            config.root.last().unwrap().display()
        );
    }

    // Ask for a download directory
    println!("\nNow, let's set up your download directory.");
    println!("This is the directory that will be scanned for files to process.");
    println!();

    loop {
        println!("Enter your download directory path:");
        let download = read_line()?;
        if download.trim().is_empty() {
            println!("You must specify a download directory.");
            continue;
        }

        let download_path = PathBuf::from(download);
        if !download_path.exists() {
            println!(
                "Warning: Directory does not exist: {}",
                download_path.display()
            );
            println!("Do you want to create it? (y/n)");
            let create = read_line()?.trim().to_lowercase();
            if create == "y" || create == "yes" {
                fs::create_dir_all(&download_path)?;
                println!("Directory created: {}", download_path.display());
            } else {
                println!("Directory not created. Please enter a different path.");
                continue;
            }
        }

        if !download_path.is_dir() {
            println!(
                "Error: Path is not a directory: {}",
                download_path.display()
            );
            println!("Please enter a directory path.");
            continue;
        }

        config.download = download_path;
        println!("Download directory set to: {}", config.download.display());
        break;
    }

    // Ask for rules
    println!("\nNow, let's create some rules for sorting files.");
    println!("Rules define how files are matched and where they are moved or copied to.");
    println!();

    loop {
        let rule = create_rule_with_wizard(&config.root)?;
        config.rules.push(rule);

        println!("Rule added successfully!");
        println!("Do you want to add another rule? (y/n)");
        let add_another = read_line()?.trim().to_lowercase();
        if add_another != "y" && add_another != "yes" {
            break;
        }
    }

    // Save the configuration
    save_config(&config, output_path)?;

    println!("\nConfiguration file created successfully: {output_path}");
    println!("You can now run File Sort with this configuration.");

    Ok(())
}

/// Creates a new rule with a wizard
///
/// This function guides the user through creating a new rule
/// by asking questions about the rule's properties.
///
/// # Arguments
/// * `root_dirs` - List of root directories
///
/// # Returns
/// * `Result<Rule>` - The created rule or an error
///
/// # Errors
/// Returns an error if the rule cannot be created
fn create_rule_with_wizard(root_dirs: &[PathBuf]) -> Result<Rule> {
    println!("Creating a new rule...");

    // Ask for rule title
    println!("Enter a title for this rule:");
    let title = read_line()?;
    if title.trim().is_empty() {
        return Err(anyhow!("Rule title cannot be empty."));
    }

    // Ask for pattern or patterns
    println!("\nYou can specify either a single pattern or multiple patterns for matching files.");
    println!("Patterns can include angle brackets to extract parts of the filename.");
    println!(
        "Example: 'document-<date>.pdf' will match 'document-2025-08-07.pdf' and extract '2025-08-07'."
    );
    println!();

    println!("Do you want to specify a single pattern or multiple patterns? (single/multiple)");
    let pattern_type = read_line()?.trim().to_lowercase();

    let (pattern, patterns) = if pattern_type == "multiple" {
        println!("Enter patterns one by one. Leave empty to finish.");
        let mut patterns_list = Vec::new();
        loop {
            println!("Enter a pattern (or leave empty to finish):");
            let pattern = read_line()?;
            if pattern.trim().is_empty() {
                if patterns_list.is_empty() {
                    println!("You must add at least one pattern.");
                    continue;
                }
                break;
            }
            patterns_list.push(pattern);
        }
        (None, Some(patterns_list))
    } else {
        println!("Enter a pattern:");
        let pattern = read_line()?;
        if pattern.trim().is_empty() {
            return Err(anyhow!("Pattern cannot be empty."));
        }
        (Some(pattern), None)
    };

    // Ask for content conditions
    println!("\nDo you want to add content-based conditions? (y/n)");
    println!(
        "Content conditions allow matching files based on properties like size, date, or content type."
    );
    let add_conditions = read_line()?.trim().to_lowercase();

    let content_conditions = if add_conditions == "y" || add_conditions == "yes" {
        Some(create_content_conditions()?)
    } else {
        None
    };

    // Ask for match_all_conditions
    let match_all_conditions = if content_conditions.is_some()
        && content_conditions.as_ref().unwrap().len() > 1
    {
        println!(
            "\nDo you want to match all conditions (AND logic) or any condition (OR logic)? (all/any)"
        );
        let match_type = read_line()?.trim().to_lowercase();
        match_type == "all"
    } else {
        true
    };

    // Ask for root index
    println!("\nSelect the root directory to use for this rule:");
    for (i, root) in root_dirs.iter().enumerate() {
        println!("{}. {}", i, root.display());
    }

    let root = loop {
        println!("Enter the number of the root directory:");
        let root_index = read_line()?;
        match root_index.trim().parse::<usize>() {
            Ok(index) if index < root_dirs.len() => break index,
            _ => println!(
                "Invalid selection. Please enter a number between 0 and {}.",
                root_dirs.len() - 1
            ),
        }
    };

    // Ask for directory
    println!("\nEnter the subdirectory within the root where files should be moved or copied to:");
    println!("(Leave empty to use the root directory directly)");
    let directory_str = read_line()?;
    let directory = if directory_str.trim().is_empty() {
        None
    } else {
        Some(PathBuf::from(directory_str))
    };

    // Ask for copy or move
    println!("\nDo you want to copy files or move them? (copy/move)");
    let copy_or_move = read_line()?.trim().to_lowercase();
    let copy = copy_or_move == "copy";

    // Create the rule
    let rule = Rule {
        title,
        pattern,
        patterns,
        content_conditions,
        match_all_conditions,
        directory,
        function: None,   // Advanced feature, not included in basic wizard
        processors: None, // Advanced feature, not included in basic wizard
        root,
        copy,
        old_pattern: String::new(), // Will be set by make_patterns
        new_pattern: String::new(), // Will be set by make_patterns
    };

    Ok(rule)
}

/// Creates content conditions with a wizard
///
/// This function guides the user through creating content conditions
/// by asking questions about the condition properties.
///
/// # Returns
/// * `Result<Vec<ContentCondition>>` - The created content conditions or an error
///
/// # Errors
/// Returns an error if the conditions cannot be created
fn create_content_conditions() -> Result<Vec<ContentCondition>> {
    let mut conditions = Vec::new();

    println!("Creating content conditions...");
    println!("You can add multiple conditions for matching files based on their properties.");

    loop {
        println!("\nSelect a property to check:");
        println!("1. File size");
        println!("2. Last modified date");
        println!("3. Creation date");
        println!("4. MIME type (content type)");
        println!("5. File content (text search)");
        println!("6. Is text file");
        println!("7. Is binary file");

        let property = loop {
            println!("Enter the number of the property:");
            let property_index = read_line()?.trim().parse::<usize>();
            match property_index {
                Ok(1) => break ContentProperty::Size,
                Ok(2) => break ContentProperty::Modified,
                Ok(3) => break ContentProperty::Created,
                Ok(4) => break ContentProperty::MimeType,
                Ok(5) => break ContentProperty::Content,
                Ok(6) => break ContentProperty::IsText,
                Ok(7) => break ContentProperty::IsBinary,
                _ => println!("Invalid selection. Please enter a number between 1 and 7."),
            }
        };

        println!("\nSelect an operator:");
        let operators = match property {
            ContentProperty::Size => {
                println!("1. Equal to (==)");
                println!("2. Not equal to (!=)");
                println!("3. Greater than (>)");
                println!("4. Less than (<)");
                println!("5. Greater than or equal to (>=)");
                println!("6. Less than or equal to (<=)");
                1..=6
            }
            ContentProperty::Modified | ContentProperty::Created => {
                println!("1. Equal to (==)");
                println!("2. Not equal to (!=)");
                println!("3. Greater than (>)");
                println!("4. Less than (<)");
                println!("5. Greater than or equal to (>=)");
                println!("6. Less than or equal to (<=)");
                1..=6
            }
            ContentProperty::MimeType => {
                println!("1. Equal to (==)");
                println!("2. Not equal to (!=)");
                println!("3. Contains");
                println!("4. Starts with");
                println!("5. Ends with");
                1..=5
            }
            ContentProperty::Content => {
                println!("1. Contains");
                println!("2. Starts with");
                println!("3. Ends with");
                println!("4. Matches regex");
                1..=4
            }
            ContentProperty::IsText | ContentProperty::IsBinary => {
                println!("1. Equal to (==)");
                println!("2. Not equal to (!=)");
                1..=2
            }
        };

        let operator = loop {
            println!("Enter the number of the operator:");
            let operator_index = read_line()?.trim().parse::<usize>();
            match operator_index {
                Ok(1) => match property {
                    ContentProperty::Content => break ConditionOperator::Contains,
                    _ => break ConditionOperator::Equal,
                },
                Ok(2) => match property {
                    ContentProperty::Content => break ConditionOperator::StartsWith,
                    _ => break ConditionOperator::NotEqual,
                },
                Ok(3) => match property {
                    ContentProperty::Size
                    | ContentProperty::Modified
                    | ContentProperty::Created => break ConditionOperator::GreaterThan,
                    ContentProperty::MimeType | ContentProperty::Content => {
                        break ConditionOperator::Contains;
                    }
                    _ => {
                        println!("Invalid selection for this property.");
                        continue;
                    }
                },
                Ok(4) => match property {
                    ContentProperty::Size
                    | ContentProperty::Modified
                    | ContentProperty::Created => break ConditionOperator::LessThan,
                    ContentProperty::MimeType => break ConditionOperator::StartsWith,
                    ContentProperty::Content => break ConditionOperator::Matches,
                    _ => {
                        println!("Invalid selection for this property.");
                        continue;
                    }
                },
                Ok(5) => match property {
                    ContentProperty::Size
                    | ContentProperty::Modified
                    | ContentProperty::Created => break ConditionOperator::GreaterThanOrEqual,
                    ContentProperty::MimeType => break ConditionOperator::EndsWith,
                    _ => {
                        println!("Invalid selection for this property.");
                        continue;
                    }
                },
                Ok(6) => match property {
                    ContentProperty::Size
                    | ContentProperty::Modified
                    | ContentProperty::Created => break ConditionOperator::LessThanOrEqual,
                    _ => {
                        println!("Invalid selection for this property.");
                        continue;
                    }
                },
                _ => println!("Invalid selection. Please enter a valid number."),
            }
        };

        println!("\nEnter the value to compare against:");
        match property {
            ContentProperty::Size => {
                println!("Enter a file size in bytes (e.g., 1024):");
            }
            ContentProperty::Modified | ContentProperty::Created => {
                println!("Enter a date in ISO 8601 format (e.g., 2025-08-07T07:23:00Z):");
            }
            ContentProperty::MimeType => {
                println!("Enter a MIME type (e.g., text/plain, image/jpeg):");
            }
            ContentProperty::Content => {
                println!("Enter the text to search for:");
            }
            ContentProperty::IsText | ContentProperty::IsBinary => {
                println!("Enter true or false:");
            }
        }

        let value = read_line()?;
        if value.trim().is_empty() {
            println!("Value cannot be empty. Please try again.");
            continue;
        }

        // Validate the value based on the property
        match property {
            ContentProperty::Size => {
                if value.trim().parse::<u64>().is_err() {
                    println!("Invalid size value. Please enter a valid number.");
                    continue;
                }
            }
            ContentProperty::Modified | ContentProperty::Created => {
                if chrono::DateTime::parse_from_rfc3339(&value).is_err() {
                    println!(
                        "Invalid date format. Please use ISO 8601 format (e.g., 2025-08-07T07:23:00Z)."
                    );
                    continue;
                }
            }
            ContentProperty::IsText | ContentProperty::IsBinary => {
                if value.trim().parse::<bool>().is_err() {
                    println!("Invalid boolean value. Please enter 'true' or 'false'.");
                    continue;
                }
            }
            _ => {}
        }

        let condition = ContentCondition {
            property,
            operator,
            value,
        };

        conditions.push(condition);

        println!("\nCondition added successfully!");
        println!("Do you want to add another condition? (y/n)");
        let add_another = read_line()?.trim().to_lowercase();
        if add_another != "y" && add_another != "yes" {
            break;
        }
    }

    Ok(conditions)
}

/// Saves a configuration to a file
///
/// # Arguments
/// * `config` - The configuration to save
/// * `output_path` - Path where the configuration file will be saved
///
/// # Returns
/// * `Result<()>` - Success or an error
///
/// # Errors
/// Returns an error if the configuration file cannot be saved
fn save_config(config: &Config, output_path: &str) -> Result<()> {
    let yaml = serde_yaml::to_string(config)?;
    let path = Path::new(output_path);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, yaml)?;
    debug!("Configuration saved to {output_path}");

    Ok(())
}

/// Reads a line from standard input
///
/// # Returns
/// * `Result<String>` - The line read or an error
///
/// # Errors
/// Returns an error if the line cannot be read
fn read_line() -> Result<String> {
    let mut input = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
