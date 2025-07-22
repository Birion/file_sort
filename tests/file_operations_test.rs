use file_sort::processor::ProcessorBuilder;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_builder() {
        // Create a processor with source and target
        let processor = ProcessorBuilder::new(Path::new("source_file.txt")).build();
        
        // Verify that the source path is set correctly
        assert_eq!(processor.source().to_str().unwrap(), "source_file.txt");
        
        // Verify that the target path is empty
        assert!(processor.target().as_os_str().is_empty());
    }
}