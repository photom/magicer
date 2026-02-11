use std::sync::Arc;
use crate::domain::entities::magic_result::MagicResult;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::services::sandbox_service::SandboxService;
use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::domain::value_objects::path::RelativePath;
use crate::domain::value_objects::request_id::RequestId;
use crate::application::errors::ApplicationError;

pub struct AnalyzePathUseCase {
    magic_repo: Arc<dyn MagicRepository>,
    sandbox: Arc<dyn SandboxService>,
}

impl AnalyzePathUseCase {
    pub fn new(magic_repo: Arc<dyn MagicRepository>, sandbox: Arc<dyn SandboxService>) -> Self {
        Self { magic_repo, sandbox }
    }

    pub async fn execute(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        path: RelativePath,
    ) -> Result<MagicResult, ApplicationError> {
        let resolved_path = self.sandbox.resolve_path(&path)?;
        
        let (mime_type, description) = self.magic_repo.analyze_file(&resolved_path).await?;
        
        Ok(MagicResult::new(
            request_id,
            filename,
            mime_type,
            description,
        ))
    }
}
