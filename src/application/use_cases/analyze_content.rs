use std::sync::Arc;
use crate::domain::entities::magic_result::MagicResult;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::domain::value_objects::request_id::RequestId;
use crate::application::errors::ApplicationError;

pub struct AnalyzeContentUseCase {
    magic_repo: Arc<dyn MagicRepository>,
}

impl AnalyzeContentUseCase {
    pub fn new(magic_repo: Arc<dyn MagicRepository>) -> Self {
        Self { magic_repo }
    }

    pub async fn execute(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        data: &[u8],
    ) -> Result<MagicResult, ApplicationError> {
        let (mime_type, description) = self.magic_repo.analyze_buffer(data, filename.as_str()).await?;
        
        Ok(MagicResult::new(
            request_id,
            filename,
            mime_type,
            description,
        ))
    }
}
