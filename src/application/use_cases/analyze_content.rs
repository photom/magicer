use crate::application::errors::ApplicationError;
use crate::domain::entities::magic_result::MagicResult;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::domain::value_objects::request_id::RequestId;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

pub struct AnalyzeContentUseCase {
    magic_repo: Arc<dyn MagicRepository>,
    analysis_timeout_secs: u64,
}

impl AnalyzeContentUseCase {
    pub fn new(magic_repo: Arc<dyn MagicRepository>, analysis_timeout_secs: u64) -> Self {
        Self {
            magic_repo,
            analysis_timeout_secs,
        }
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

        let (mime_type, description) = timeout(
            Duration::from_secs(self.analysis_timeout_secs),
            self.magic_repo.analyze_buffer(data, filename.as_str()),
        )
        .await
        .map_err(|_| ApplicationError::Timeout)??;

        Ok(MagicResult::new(
            request_id,
            filename,
            mime_type,
            description,
        ))
    }

    pub async fn execute_from_file(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        path: &Path,
    ) -> Result<MagicResult, ApplicationError> {
        let (mime_type, description) = timeout(
            Duration::from_secs(self.analysis_timeout_secs),
            self.magic_repo.analyze_file(path),
        )
        .await
        .map_err(|_| ApplicationError::Timeout)??;

        Ok(MagicResult::new(
            request_id,
            filename,
            mime_type,
            description,
        ))
    }
}
