use crate::domain::errors::ValidationError;
use crate::domain::services::sandbox_service::SandboxService;
use crate::domain::value_objects::path::RelativePath;
use std::path::PathBuf;

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

        // Ensure the path is within base_dir
        // Since RelativePath already prevents '..', a simple join should stay within base_dir
        // unless base_dir itself is malicious or if there are symlinks.
        // For production, we should use canonicalize() but it requires file to exist.

        if !full_path.starts_with(&self.base_dir) {
            return Err(ValidationError::PathTraversal);
        }

        Ok(full_path)
    }
}
