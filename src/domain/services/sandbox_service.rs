use std::path::PathBuf;
use crate::domain::value_objects::path::RelativePath;
use crate::domain::errors::ValidationError;

pub trait SandboxService: Send + Sync {
    fn resolve_path(&self, path: &RelativePath) -> Result<PathBuf, ValidationError>;
}
