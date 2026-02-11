use magicer::infrastructure::filesystem::temp_file_handler::TempFileHandler;
use std::fs;
use std::io::Read;
use tempfile::tempdir;

#[tokio::test]
async fn test_temp_file_creation_and_cleanup() {
    let dir = tempdir().unwrap();
    let temp_dir_path = dir.path().to_path_buf();
    let data = b"test data";
    let path;

    {
        let handler = TempFileHandler::create_temp_file(data, &temp_dir_path).expect("Failed to create temp file");
        path = handler.path().to_path_buf();

        assert!(path.exists());
        let mut file_data = Vec::new();
        fs::File::open(&path).unwrap().read_to_end(&mut file_data).unwrap();
        assert_eq!(file_data, data);
    }

    // After drop, file should be gone
    assert!(!path.exists());
}
