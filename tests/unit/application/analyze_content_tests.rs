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

    fn analyze_file<'a>(&'a self, _path: &'a Path) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        Box::pin(async {
            Ok((MimeType::try_from("application/octet-stream").unwrap(), "data".to_string()))
        })
    }
}

#[tokio::test]
async fn test_analyze_content_success() {
    let repo: Arc<dyn MagicRepository> = Arc::new(FakeMagicRepo);
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let use_case = AnalyzeContentUseCase::new(repo, config);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    let data = b"%PDF-1.4";
    
    let result = use_case.execute(request_id, filename, data).await.unwrap();
    
    assert_eq!(result.mime_type().as_str(), "application/pdf");
    assert_eq!(result.description(), "PDF document");
}

#[tokio::test]
async fn test_analyze_content_large_success() {
    let repo: Arc<dyn MagicRepository> = Arc::new(FakeMagicRepo);
    let mut config = magicer::infrastructure::config::server_config::ServerConfig::default();
    config.analysis.large_file_threshold_mb = 0; // Force large file path
    let config = Arc::new(config);
    
    let use_case = AnalyzeContentUseCase::new(repo, config);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.dat").unwrap();
    let data = b"some large data content";
    
    let result = use_case.execute(request_id, filename, data).await.unwrap();
    
    assert_eq!(result.mime_type().as_str(), "application/pdf"); // Fake repo returns pdf for everything
}

#[tokio::test]
async fn test_analyze_content_empty_rejected() {
    let repo: Arc<dyn MagicRepository> = Arc::new(FakeMagicRepo);
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let use_case = AnalyzeContentUseCase::new(repo, config);
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
    fn analyze_file<'a>(&'a self, _path: &'a Path) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        Box::pin(async { Err(MagicError::AnalysisFailed("forced failure".to_string())) })
    }
}

#[tokio::test]
async fn test_analyze_content_repository_failure() {
    let repo: Arc<dyn MagicRepository> = Arc::new(FailingMagicRepo);
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let use_case = AnalyzeContentUseCase::new(repo, config);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    
    let result = use_case.execute(request_id, filename, b"some data").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Analysis failed: forced failure"));
}
