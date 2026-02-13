use crate::domain::errors::MagicError;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::value_objects::mime_type::MimeType;
use crate::infrastructure::magic::ffi::*;
use crate::infrastructure::magic::wrapper::MagicCookie;
use futures_util::future::BoxFuture;
use std::path::Path;
use std::sync::Arc;

use crate::infrastructure::filesystem::mmap::MmapHandler;

pub struct LibmagicRepository {
    cookie: Arc<MagicCookie>,
    mmap_fallback_enabled: bool,
}

impl LibmagicRepository {
    pub fn new(mmap_fallback_enabled: bool) -> Result<Self, MagicError> {
        let cookie = MagicCookie::open(MAGIC_MIME_TYPE)?;
        cookie.load(None)?; // Load default database
        Ok(Self {
            cookie: Arc::new(cookie),
            mmap_fallback_enabled,
        })
    }
}

impl MagicRepository for LibmagicRepository {
    fn analyze_buffer<'a>(
        &'a self,
        data: &'a [u8],
        _filename: &'a str,
    ) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        let cookie = self.cookie.clone();
        let data_vec = data.to_vec();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let mime = cookie.buffer(&data_vec)?;
                Ok((
                    MimeType::try_from(mime.as_str()).map_err(|_| {
                        MagicError::AnalysisFailed("Invalid MIME returned".to_string())
                    })?,
                    mime,
                ))
            })
            .await
            .map_err(|e| MagicError::AnalysisFailed(e.to_string()))?
        })
    }

    fn analyze_file<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        let cookie = self.cookie.clone();
        let path_owned = path.to_path_buf();
        let fallback = self.mmap_fallback_enabled;

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let file = std::fs::File::open(&path_owned)
                    .map_err(|e| MagicError::FileNotFound(e.to_string()))?;

                let result = match MmapHandler::new(&file) {
                    Ok(mmap) => cookie.buffer(mmap.as_slice()),
                    Err(e) if fallback => {
                        tracing::warn!("Mmap failed, falling back to magic_file: {}", e);
                        cookie.file(path_owned.to_str().unwrap())
                    }
                    Err(e) => Err(MagicError::AnalysisFailed(format!(
                        "Failed to mmap file: {}",
                        e
                    ))),
                }?;

                Ok((
                    MimeType::try_from(result.as_str()).map_err(|_| {
                        MagicError::AnalysisFailed("Invalid MIME returned".to_string())
                    })?,
                    result,
                ))
            })
            .await
            .map_err(|e| MagicError::AnalysisFailed(e.to_string()))?
        })
    }
}
