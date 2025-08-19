use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;
use tempfile::tempdir;

#[test]
fn test_parallel_processing() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let test_dir = temp_dir.path().join("test_files");

    // Create the test directory
    create_dir_all(&test_dir).expect("Failed to create test directory");

    // Create test files
    const NUM_FILES: usize = 100;
    for i in 0..NUM_FILES {
        let file_path = test_dir.join(format!("test_file_{i}.txt"));
        let mut file = File::create(&file_path).expect("Failed to create test file");
        writeln!(file, "Test content for file {i}").expect("Failed to write to test file");
    }

    // Process files sequentially
    let sequential_start = Instant::now();
    let sequential_count = process_files_sequential(&test_dir);
    let sequential_duration = sequential_start.elapsed();

    // Process files in parallel
    let parallel_start = Instant::now();
    let parallel_count = process_files_parallel(&test_dir);
    let parallel_duration = parallel_start.elapsed();

    println!("Sequential processing: {sequential_count} files in {sequential_duration:?}");
    println!("Parallel processing: {parallel_count} files in {parallel_duration:?}");

    // Verify that both methods processed the same number of files
    assert_eq!(sequential_count, NUM_FILES);
    assert_eq!(parallel_count, NUM_FILES);

    // Verify that parallel processing is faster (this might not always be true for small tests),
    // but we're mainly checking that it works correctly
    println!(
        "Speedup: {:.2}x",
        sequential_duration.as_secs_f64() / parallel_duration.as_secs_f64()
    );
}

// Process files sequentially
fn process_files_sequential(dir: &PathBuf) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata()
                && metadata.is_file()
            {
                // Simulate some work
                std::thread::sleep(std::time::Duration::from_millis(10));
                count += 1;
            }
        }
    }
    count
}

// Process files in parallel
fn process_files_parallel(dir: &PathBuf) -> usize {
    let count = Arc::new(AtomicUsize::new(0));

    if let Ok(entries) = std::fs::read_dir(dir) {
        let entries: Vec<_> = entries.filter_map(Result::ok).collect();

        entries.par_iter().for_each(|entry| {
            if let Ok(metadata) = entry.metadata()
                && metadata.is_file()
            {
                // Simulate some work
                std::thread::sleep(std::time::Duration::from_millis(10));
                count.fetch_add(1, Ordering::Relaxed);
            }
        });
    }

    count.load(Ordering::Relaxed)
}
