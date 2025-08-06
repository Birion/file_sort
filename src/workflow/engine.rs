//! Workflow engine
//!
//! This module contains the engine that orchestrates the workflow steps.

use std::path::PathBuf;

use anyhow::Result;
use log::{debug, error, info};

use crate::config::{load_config, prepare_rules, read_or_create};
use crate::discovery::{match_file_against_rules, scan_directory};
use crate::file_ops::{convert_file_format, perform_file_action};
use crate::path_gen::{apply_transformative_function, generate_destination_path};

use super::context::{OperationType, PlannedOperation, WorkflowContext};

/// Options for processing files
#[derive(Debug, Clone)]
pub struct ProcessingOptions {
    /// Path to the configuration file
    pub config_path: PathBuf,
    /// Whether to actually perform file operations (true) or just simulate them (false)
    pub dry_run: bool,
}

/// Processes files based on the configuration
///
/// This function orchestrates the workflow steps:
/// 1. Read the configuration
/// 2. Get the list of files and check if any of the configuration rules applies
/// 3. If the rule has a defined transformative function, use it to generate the new destination filename
/// 4. Apply any file operations to the source file (move and/or copy)
/// 5. If the rule has a defined conversion function, apply it before any other file operations
///
/// # Arguments
/// * `options` - Options for processing files
///
/// # Returns
/// * `Result<WorkflowContext>` - The workflow context with statistics or an error
///
/// # Errors
/// * Returns an error if any step fails
pub fn process_files(options: ProcessingOptions) -> Result<WorkflowContext> {
    // Step 1: Read the configuration
    let config_file_path = read_or_create(options.config_path)?;
    let mut config = load_config(config_file_path)?;

    // Prepare the rules
    prepare_rules(&mut config.rules)?;

    // Create the workflow context
    let mut context = WorkflowContext::new(config.clone(), options.dry_run);

    // Step 2: Get the list of files from the download directory
    let files = scan_directory(&config.download)?;

    if files.is_empty() {
        info!("No files found in the download folder");
        return Ok(context);
    }

    info!(
        "Processing {} files{}...",
        files.len(),
        if options.dry_run { " (dry run)" } else { "" }
    );

    // Process each file
    for file_info in files {
        debug!("Processing file: {}", file_info.path.display());
        context.increment_files_processed();

        // Step 2: Check if any of the configuration rules applies
        let matches = match_file_against_rules(&file_info, &config.rules)?;

        for match_result in matches {
            context.increment_files_matched();

            // Step 3: If the rule has a defined transformative function, use it to generate the new destination filename
            let transform_result = match apply_transformative_function(
                &match_result.file_info.path,
                &config.root,
                &match_result.rule,
            ) {
                Ok(result) => result,
                Err(e) => {
                    error!("Failed to apply transformative function: {e}");
                    context.increment_errors();
                    continue;
                }
            };

            // Generate the destination path
            let path_result = match generate_destination_path(&transform_result, !options.dry_run) {
                Ok(result) => result,
                Err(e) => {
                    error!("Failed to generate destination path: {e}");
                    context.increment_errors();
                    continue;
                }
            };

            // Step 5: If the rule has a defined conversion function, apply it before any other file operations
            let conversion_result = match convert_file_format(&path_result, !options.dry_run) {
                Ok(result) => {
                    if result.source_path != path_result.source_path
                        || result.target_path != path_result.target_path
                    {
                        context.increment_files_converted();

                        // Track planned conversion operation in dry-run mode
                        if options.dry_run {
                            // Determine source and target formats from file extensions
                            let source_ext = result
                                .source_path
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("unknown");
                            let target_ext = result
                                .target_path
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("unknown");

                            context.add_planned_operation(PlannedOperation {
                                source: result.source_path.clone(),
                                destination: result.target_path.clone(),
                                operation_type: OperationType::Convert(
                                    source_ext.to_string(),
                                    target_ext.to_string(),
                                ),
                                rule_title: match_result.rule.title.clone(),
                            });
                        }
                    }
                    result
                }
                Err(e) => {
                    error!("Failed to convert file format: {e}");
                    context.increment_errors();
                    continue;
                }
            };

            // Step 4: Apply any file operations to the source file (move and/or copy)
            let action_result = match perform_file_action(&conversion_result, !options.dry_run) {
                Ok(result) => result,
                Err(e) => {
                    error!("Failed to perform file action: {e}");
                    context.increment_errors();
                    continue;
                }
            };

            // Update statistics
            if action_result.success {
                if match_result.rule.copy {
                    context.increment_files_copied();

                    // Track planned copy operation in dry-run mode
                    if options.dry_run {
                        context.add_planned_operation(PlannedOperation {
                            source: action_result.source_path.clone(),
                            destination: action_result.target_path.clone(),
                            operation_type: OperationType::Copy,
                            rule_title: match_result.rule.title.clone(),
                        });
                    }
                } else {
                    context.increment_files_moved();

                    // Track planned move operation in dry-run mode
                    if options.dry_run {
                        context.add_planned_operation(PlannedOperation {
                            source: action_result.source_path.clone(),
                            destination: action_result.target_path.clone(),
                            operation_type: OperationType::Move,
                            rule_title: match_result.rule.title.clone(),
                        });
                    }
                }
            }
        }
    }

    info!(
        "Finished processing {} files",
        context.stats.files_processed
    );

    // Display detailed output for planned operations in dry-run mode
    if options.dry_run && !context.planned_operations.is_empty() {
        println!("\nDetailed plan of operations:");
        println!("===========================");

        // Group operations by type for better readability
        let mut move_operations = Vec::new();
        let mut copy_operations = Vec::new();
        let mut convert_operations = Vec::new();

        for op in &context.planned_operations {
            match op.operation_type {
                OperationType::Move => move_operations.push(op),
                OperationType::Copy => copy_operations.push(op),
                OperationType::Convert(_, _) => convert_operations.push(op),
            }
        }

        // Display move operations
        if !move_operations.is_empty() {
            println!("\nFiles to be moved:");
            println!("-----------------");
            for op in move_operations.iter() {
                println!("Rule: {}", op.rule_title);
                println!("  From: {}", op.source.display());
                println!("  To:   {}", op.destination.display());
            }
        }

        // Display copy operations
        if !copy_operations.is_empty() {
            println!("\nFiles to be copied:");
            println!("------------------");
            for op in copy_operations.iter() {
                println!("Rule: {}", op.rule_title);
                println!("  From: {}", op.source.display());
                println!("  To:   {}", op.destination.display());
            }
        }

        // Display convert operations
        if !convert_operations.is_empty() {
            println!("\nFiles to be converted:");
            println!("--------------------");
            for op in convert_operations.iter() {
                if let OperationType::Convert(from, to) = &op.operation_type {
                    println!("Rule: {}", op.rule_title);
                    println!("  From: {} ({})", op.source.display(), from);
                    println!("  To:   {} ({})", op.destination.display(), to);
                }
            }
        }

        println!("\nSummary:");
        println!("--------");
        println!("  Files to be moved:    {}", move_operations.len());
        println!("  Files to be copied:   {}", copy_operations.len());
        println!("  Files to be converted: {}", convert_operations.len());
        println!(
            "  Total operations:     {}",
            context.planned_operations.len()
        );
        println!("\nRun without --dry flag to execute these operations.");
    }

    Ok(context)
}
