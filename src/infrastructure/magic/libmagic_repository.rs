use std::path::Path;
use std::sync::Arc;
use futures_util::future::BoxFuture;
use crate::domain::errors::MagicError;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::value_objects::mime_type::MimeType;
use crate::infrastructure::magic::wrapper::MagicCookie;
use crate::infrastructure::magic::ffi::*;

pub struct LibmagicRepository {
    cookie: Arc<MagicCookie>,
}

impl LibmagicRepository {
    pub fn new() -> Result<Self, MagicError> {
        let cookie = MagicCookie::open(MAGIC_MIME_TYPE)?;
        cookie.load(None)?; // Load default database
        Ok(Self { cookie: Arc::new(cookie) })
    }
}

impl MagicRepository for LibmagicRepository {
    fn analyze_buffer<'a>(&'a self, data: &'a [u8], _filename: &'a str) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        let cookie = self.cookie.clone();
        let data_vec = data.to_vec();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let mime = cookie.buffer(&data_vec)?;
                Ok((
                    MimeType::try_from(mime.as_str()).map_err(|_| MagicError::AnalysisFailed("Invalid MIME returned".to_string()))?,
                    mime
                ))
            }).await.map_err(|e| MagicError::AnalysisFailed(e.to_string()))?
        })
    }

    fn analyze_file<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        let cookie = self.cookie.clone();
        let path_owned = path.to_path_buf();
        Box::pin(async move {
            let path_str = path_owned.to_str().ok_or_else(|| MagicError::FileNotFound("Invalid path".to_string()))?.to_string();
            tokio::task::spawn_blocking(move || {
                let mime = cookie.file(&path_str)?;
                Ok((
                    MimeType::try_from(mime.as_str()).map_err(|_| MagicError::AnalysisFailed("Invalid MIME returned".to_string()))?,
                    mime
                ))
            }).await.map_err(|e| MagicError::AnalysisFailed(e.to_string()))?
        })
    }
}
