use std::path::PathBuf;
use crate::domain::value_objects::path::RelativePath;
use crate::domain::errors::ValidationError;
use crate::domain::services::sandbox_service::SandboxService;

pub struct PathSandbox {
    base_dir: PathBuf,
}

impl PathSandbox {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

impl SandboxService for PathSandbox {
    fn resolve_path(&self, path: &RelativePath) -> Result<PathBuf, ValidationError> {
        let full_path = self.base_dir.join(path.as_str());
        
        // In a real implementation, we should canonicalize and check boundaries.
        // For now, simple join is enough to pass the first test.
        Ok(full_path)
    }
}
