use std::sync::Arc;
use std::path::Path;
use futures_util::future::BoxFuture;
use magicer::application::use_cases::analyze_content::AnalyzeContentUseCase;
use magicer::domain::repositories::magic_repository::MagicRepository;
use magicer::domain::value_objects::request_id::RequestId;
use magicer::domain::value_objects::filename::WindowsCompatibleFilename;
use magicer::domain::value_objects::mime_type::MimeType;
use magicer::domain::errors::MagicError;
use magicer::application::errors::ApplicationError;

struct FakeMagicRepo;

impl MagicRepository for FakeMagicRepo {
    fn analyze_buffer<'a>(&'a self, _data: &'a [u8], _filename: &'a str) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        Box::pin(async {
            Ok((MimeType::try_from("application/pdf").unwrap(), "PDF document".to_string()))
        })
    }
}

use magicer::domain::services::temp_storage::{TempStorageService, TemporaryFile};
use async_trait::async_trait;

struct FakeTemporaryFile {
    path: std::path::PathBuf,
}

#[async_trait]
impl TemporaryFile for FakeTemporaryFile {
    async fn write(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.path)?;
        file.write_all(data)
    }

    async fn sync(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

struct FakeTempStorage;

#[async_trait]
impl TempStorageService for FakeTempStorage {
    async fn create_temp_file(&self) -> Result<Box<dyn TemporaryFile>, std::io::Error> {
        let path = std::env::temp_dir().join(format!("fake_temp_{}.tmp", uuid::Uuid::new_v4()));
        Ok(Box::new(FakeTemporaryFile { path }))
    }
}

#[tokio::test]
async fn test_analyze_to_temp_file_success() {
    let repo: Arc<dyn MagicRepository> = Arc::new(FakeMagicRepo);
    let temp_storage: Arc<dyn TempStorageService> = Arc::new(FakeTempStorage);
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let use_case = AnalyzeContentUseCase::new(repo, temp_storage, config);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    
    let data = b"%PDF-1.4";
    let stream = futures_util::stream::iter(vec![Ok::<_, std::io::Error>(bytes::Bytes::from_static(data))]);
    
    let result = use_case.analyze_to_temp_file(request_id, filename, stream).await.unwrap();
    
    assert_eq!(result.mime_type().as_str(), "application/pdf");
}

#[tokio::test]
async fn test_analyze_in_memory_success() {
    let repo: Arc<dyn MagicRepository> = Arc::new(FakeMagicRepo);
    let temp_storage: Arc<dyn TempStorageService> = Arc::new(FakeTempStorage);
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let use_case = AnalyzeContentUseCase::new(repo, temp_storage, config);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    
    let data = b"%PDF-1.4";
    let stream = futures_util::stream::iter(vec![Ok::<_, std::io::Error>(bytes::Bytes::from_static(data))]);
    
    let result = use_case.analyze_in_memory(request_id, filename, stream).await.unwrap();
    
    assert_eq!(result.mime_type().as_str(), "application/pdf");
}

#[tokio::test]
async fn test_analyze_content_success() {
    let repo: Arc<dyn MagicRepository> = Arc::new(FakeMagicRepo);
    let temp_storage: Arc<dyn TempStorageService> = Arc::new(FakeTempStorage);
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let use_case = AnalyzeContentUseCase::new(repo, temp_storage, config);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    let data = b"%PDF-1.4";
    
    let result = use_case.execute(request_id, filename, data).await.unwrap();
    
    assert_eq!(result.mime_type().as_str(), "application/pdf");
    assert_eq!(result.description(), "PDF document");
}

#[tokio::test]
async fn test_execute_from_file_success() {
    let repo: Arc<dyn MagicRepository> = Arc::new(FakeMagicRepo);
    let temp_storage: Arc<dyn TempStorageService> = Arc::new(FakeTempStorage);
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let use_case = AnalyzeContentUseCase::new(repo, temp_storage, config);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    
    // Create a temporary file for the test
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("test_execute_from_file.pdf");
    std::fs::write(&temp_path, b"%PDF-1.4").unwrap();
    
    let result = use_case.execute_from_file(request_id, filename, &temp_path).await.unwrap();
    
    assert_eq!(result.mime_type().as_str(), "application/pdf");
    
    let _ = std::fs::remove_file(temp_path);
}

#[tokio::test]
async fn test_analyze_content_empty_rejected() {
    let repo: Arc<dyn MagicRepository> = Arc::new(FakeMagicRepo);
    let temp_storage: Arc<dyn TempStorageService> = Arc::new(FakeTempStorage);
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let use_case = AnalyzeContentUseCase::new(repo, temp_storage, config);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    let data = b"";
    
    // Note: Our implementation doesn't currently explicitly reject empty content in use case, 
    // it just passes it to libmagic. Let's see if we should add a check.
    // Based on the test plan, it SHOULD be rejected.
    let result = use_case.execute(request_id, filename, data).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ApplicationError::BadRequest(_)));
}

struct FailingMagicRepo;
impl MagicRepository for FailingMagicRepo {
    fn analyze_buffer<'a>(&'a self, _data: &'a [u8], _filename: &'a str) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        Box::pin(async { Err(MagicError::AnalysisFailed("forced failure".to_string())) })
    }
}

#[tokio::test]
async fn test_analyze_content_repository_failure() {
    let repo: Arc<dyn MagicRepository> = Arc::new(FailingMagicRepo);
    let temp_storage: Arc<dyn TempStorageService> = Arc::new(FakeTempStorage);
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let use_case = AnalyzeContentUseCase::new(repo, temp_storage, config);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    
    let result = use_case.execute(request_id, filename, b"some data").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Analysis failed: forced failure"));
}

struct SlowMagicRepo;
impl MagicRepository for SlowMagicRepo {
    fn analyze_buffer<'a>(&'a self, _data: &'a [u8], _filename: &'a str) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        Box::pin(async {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            Ok((MimeType::try_from("application/pdf").unwrap(), "PDF document".to_string()))
        })
    }
}

#[tokio::test]
async fn test_analyze_content_timeout() {
    let repo: Arc<dyn MagicRepository> = Arc::new(SlowMagicRepo);
    let temp_storage: Arc<dyn TempStorageService> = Arc::new(FakeTempStorage);
    let mut config_val = magicer::infrastructure::config::server_config::ServerConfig::default();
    config_val.server.timeouts.analysis_timeout_secs = 1;
    let config = Arc::new(config_val);
    let use_case = AnalyzeContentUseCase::new(repo, temp_storage, config);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    
    let result = use_case.execute(request_id, filename, b"some data").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), axum::http::StatusCode::GATEWAY_TIMEOUT);
}
