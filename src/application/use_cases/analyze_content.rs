use crate::application::errors::ApplicationError;
use crate::domain::entities::magic_result::MagicResult;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::domain::value_objects::request_id::RequestId;
use crate::infrastructure::config::server_config::ServerConfig;
use crate::infrastructure::filesystem::mmap::MmapHandler;
use crate::infrastructure::filesystem::temp_file_handler::TempFileHandler;
use std::path::Path;
use std::sync::Arc;

pub struct AnalyzeContentUseCase {
    magic_repo: Arc<dyn MagicRepository>,
    config: Arc<ServerConfig>,
}

impl AnalyzeContentUseCase {
    pub fn new(magic_repo: Arc<dyn MagicRepository>, config: Arc<ServerConfig>) -> Self {
        Self { magic_repo, config }
    }

    pub async fn execute(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        data: &[u8],
    ) -> Result<MagicResult, ApplicationError> {
        if data.is_empty() {
            return Err(ApplicationError::BadRequest(
                "Content cannot be empty".to_string(),
            ));
        }

        let threshold = self.config.analysis.large_file_threshold_mb * 1024 * 1024;

        let (mime_type, description) = if data.len() < threshold {
            self.magic_repo
                .analyze_buffer(data, filename.as_str())
                .await?
        } else {
            let temp_dir = Path::new(&self.config.analysis.temp_dir);
            let temp_file = TempFileHandler::create_temp_file(data, temp_dir).map_err(|e| {
                ApplicationError::InternalError(format!("Failed to create temp file: {}", e))
            })?;

            let mmap =
                MmapHandler::new(&std::fs::File::open(temp_file.path()).unwrap()).map_err(|e| {
                    ApplicationError::InternalError(format!("Failed to mmap temp file: {}", e))
                })?;

            let result = self
                .magic_repo
                .analyze_buffer(mmap.as_slice(), filename.as_str())
                .await;

            // Cleanup happens on drop of temp_file and mmap
            result?
        };

        Ok(MagicResult::new(
            request_id,
            filename,
            mime_type,
            description,
        ))
    }
}
