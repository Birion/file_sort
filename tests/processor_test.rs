use file_sort::processor::{Processor, ProcessorBuilder};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_builder_new() {
        // Create a test file path
        let file_path = Path::new("test_file.txt");
        
        // Create a new ProcessorBuilder
        let builder = ProcessorBuilder::new(file_path);
        
        // Build the Processor
        let processor = builder.build();
        
        // Verify that the source path is set correctly
        assert_eq!(processor.source(), &PathBuf::from("test_file.txt"));
        
        // Verify that the target path is empty
        assert_eq!(processor.target(), &PathBuf::new());
    }

    #[test]
    fn test_processor_builder_with_source() {
        // Create a test file path
        let file_path = Path::new("test_file.txt");
        
        // Create a new ProcessorBuilder with a custom source
        let custom_source = PathBuf::from("custom_source.txt");
        let builder = ProcessorBuilder::new(file_path).source(custom_source.clone());
        
        // Build the Processor
        let processor = builder.build();
        
        // Verify that the source path is set to the custom source
        assert_eq!(processor.source(), &custom_source);
        
        // Verify that the target path is empty
        assert_eq!(processor.target(), &PathBuf::new());
    }

    #[test]
    fn test_processor_builder_with_target() {
        // Create a test file path
        let file_path = Path::new("test_file.txt");
        
        // Create a new ProcessorBuilder with a custom target
        let custom_target = PathBuf::from("custom_target.txt");
        let builder = ProcessorBuilder::new(file_path).target(custom_target.clone());
        
        // Build the Processor
        let processor = builder.build();
        
        // Verify that the source path is set correctly
        assert_eq!(processor.source(), &PathBuf::from("test_file.txt"));
        
        // Verify that the target path is set to the custom target
        assert_eq!(processor.target(), &custom_target);
    }

    #[test]
    fn test_processor_builder_with_source_and_target() {
        // Create a test file path
        let file_path = Path::new("test_file.txt");
        
        // Create a new ProcessorBuilder with custom source and target
        let custom_source = PathBuf::from("custom_source.txt");
        let custom_target = PathBuf::from("custom_target.txt");
        let builder = ProcessorBuilder::new(file_path)
            .source(custom_source.clone())
            .target(custom_target.clone());
        
        // Build the Processor
        let processor = builder.build();
        
        // Verify that the source path is set to the custom source
        assert_eq!(processor.source(), &custom_source);
        
        // Verify that the target path is set to the custom target
        assert_eq!(processor.target(), &custom_target);
    }

    #[test]
    fn test_processor_builder_method_chaining() {
        // Create a test file path
        let file_path = Path::new("test_file.txt");
        
        // Create a new ProcessorBuilder with method chaining
        let custom_source = PathBuf::from("custom_source.txt");
        let custom_target = PathBuf::from("custom_target.txt");
        
        // Test that method chaining works in any order
        let processor1 = ProcessorBuilder::new(file_path)
            .source(custom_source.clone())
            .target(custom_target.clone())
            .build();
            
        let processor2 = ProcessorBuilder::new(file_path)
            .target(custom_target.clone())
            .source(custom_source.clone())
            .build();
        
        // Verify that both processors have the same source and target
        assert_eq!(processor1.source(), processor2.source());
        assert_eq!(processor1.target(), processor2.target());
    }

    #[test]
    fn test_processor_builder_from_processor() {
        // Create a test file path
        let file_path = Path::new("test_file.txt");
        
        // Create a Processor using the builder
        let processor = Processor::builder(file_path).build();
        
        // Verify that the source path is set correctly
        assert_eq!(processor.source(), &PathBuf::from("test_file.txt"));
        
        // Verify that the target path is empty
        assert_eq!(processor.target(), &PathBuf::new());
    }
}