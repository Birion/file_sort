use file_sort::utils::{clean_pattern, extract_pattern, process_date, process_pattern};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_pattern() {
        // Test with angle brackets
        let result = clean_pattern("<pattern>").unwrap();
        assert_eq!(result, "pattern");

        // Test with multiple angle brackets
        let result = clean_pattern("<pat<ter>n>").unwrap();
        assert_eq!(result, "pattern");

        // Test with no angle brackets
        let result = clean_pattern("pattern").unwrap();
        assert_eq!(result, "pattern");

        // Test with empty string
        let result = clean_pattern("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_extract_pattern() {
        // Test with angle brackets
        let result = extract_pattern("<pattern>").unwrap();
        assert_eq!(result, "pattern");

        // Test with no angle brackets
        let result = extract_pattern("pattern").unwrap();
        assert_eq!(result, "pattern");

        // Test with empty angle brackets
        let result = extract_pattern("<>").unwrap();
        assert_eq!(result, "");

        // Test with nested angle brackets (should only extract the innermost content)
        let result = extract_pattern("<outer<inner>>").unwrap();
        assert_eq!(result, "inner>");
    }

    #[test]
    fn test_process_date() {
        // Test with valid timestamp and splitter
        let mut destination = "1626912000_filename.txt".to_string();
        let fmt = "%Y-%m-%d";
        let splitter = "_";
        let merger = Some(" ".to_string());

        process_date(&mut destination, fmt, splitter, &merger).unwrap();
        assert_eq!(destination, "2021-07-22 filename.txt");

        // Test with different format
        let mut destination = "1626912000_filename.txt".to_string();
        let fmt = "%d/%m/%Y";
        let splitter = "_";
        let merger = Some("-".to_string());

        process_date(&mut destination, fmt, splitter, &merger).unwrap();
        assert_eq!(destination, "22/07/2021-filename.txt");

        // Test with invalid timestamp (should return an error)
        let mut destination = "invalid_filename.txt".to_string();
        let fmt = "%Y-%m-%d";
        let splitter = "_";
        let merger = Some(" ".to_string());

        let result = process_date(&mut destination, fmt, splitter, &merger);
        assert!(result.is_err());

        // Test with missing splitter (should return an error)
        let mut destination = "1626912000filename.txt".to_string();
        let fmt = "%Y-%m-%d";
        let splitter = "_";
        let merger = Some(" ".to_string());

        let result = process_date(&mut destination, fmt, splitter, &merger);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_pattern() {
        // Test with valid pattern and replacement
        let mut destination = "test_filename.txt".to_string();
        let pattern = "test";
        let replacement = Some("replaced".to_string());

        process_pattern(&mut destination, pattern, &replacement).unwrap();
        assert_eq!(destination, "replaced_filename.txt");

        // Test with regex pattern
        let mut destination = "test123_filename.txt".to_string();
        let pattern = r"test\d+";
        let replacement = Some("replaced".to_string());

        process_pattern(&mut destination, pattern, &replacement).unwrap();
        assert_eq!(destination, "replaced_filename.txt");

        // Test with no replacement (should not change the string)
        let mut destination = "test_filename.txt".to_string();
        let pattern = "test";
        let replacement = None;

        process_pattern(&mut destination, pattern, &replacement).unwrap();
        assert_eq!(destination, "test_filename.txt");

        // Test with invalid regex pattern (should return an error)
        let mut destination = "test_filename.txt".to_string();
        let pattern = "["; // Invalid regex pattern
        let replacement = Some("replaced".to_string());

        let result = process_pattern(&mut destination, pattern, &replacement);
        assert!(result.is_err());
    }
}
