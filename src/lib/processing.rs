use std::error::Error;
use std::fs::rename;
use std::path::PathBuf;

use colored::Colorize;
use regex::Regex;

use super::structs::{Config, Processor};

pub fn process(file: &PathBuf, config: &Config) -> Result<(), Box<dyn Error>> {
    let mut processor: Processor = Processor::new(file);

    for mapping in &config.mappings {
        let map_pattern: Regex = Regex::new(mapping.old_pattern.as_str()).unwrap();
        if map_pattern.is_match(&processor.filename()) {
            processor.make_target_dir(&config.root, &mapping.directory);

            let source: PathBuf = config.download.join(processor.filename());
            let target: PathBuf = processor.make_dst(&mapping.new_pattern, &config.root, &mapping);
            let new_filename: &str = target.file_name().unwrap().to_str().unwrap();
            println!(
                "{file} found! Applying setup for {title}.",
                file = processor.filename().bold(),
                title = mapping.title.bold().blue(),
            );
            if new_filename != processor.filename()
            {
                println!("New filename: {}", new_filename.bold().red())
            }
            println!();

            let _ = rename(source, target);
        }
    }

    Ok(())
}
