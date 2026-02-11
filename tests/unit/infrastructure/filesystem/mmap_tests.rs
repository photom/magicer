use magicer::infrastructure::filesystem::mmap::MmapHandler;
use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_mmap_readonly_enforced() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"hello world").unwrap();
    
    let file = File::open(temp_file.path()).unwrap();
    let handler = MmapHandler::new(&file).expect("Failed to mmap");
    
    let slice = handler.as_slice();
    assert_eq!(slice, b"hello world");
    
    // Attempting to write to this slice would be a compile error because it's &[u8].
    // To test runtime enforcement, one could use unsafe but that might just crash the test.
}

#[test]
fn test_mmap_empty_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let file = File::open(temp_file.path()).unwrap();
    let handler = MmapHandler::new(&file).expect("Failed to mmap empty file");
    assert_eq!(handler.as_slice().len(), 0);
}
