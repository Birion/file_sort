use file_sort::processor::ProcessorBuilder;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_source_target() {
        // Create a processor with source and target
        let processor = ProcessorBuilder::new(Path::new("source_file.txt"))
            .target(PathBuf::from("target_dir/target_file.txt"))
            .build();

        // Verify that the source path is set correctly
        assert_eq!(processor.source(), &PathBuf::from("source_file.txt"));

        // Verify that the target path is set correctly
        assert_eq!(
            processor.target(),
            &PathBuf::from("target_dir/target_file.txt")
        );
    }
}
