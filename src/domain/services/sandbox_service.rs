use crate::domain::errors::ValidationError;
use crate::domain::value_objects::path::RelativePath;
use std::path::PathBuf;

pub trait SandboxService: Send + Sync {
    fn resolve_path(&self, path: &RelativePath) -> Result<PathBuf, ValidationError>;
}
