use crate::domain::errors::MagicError;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::value_objects::mime_type::MimeType;
use futures_util::future::BoxFuture;
use std::path::Path;

pub struct FakeMagicRepository;

impl FakeMagicRepository {
    pub fn new() -> Result<Self, MagicError> {
        Ok(Self)
    }
}

impl MagicRepository for FakeMagicRepository {
    fn analyze_buffer<'a>(
        &'a self,
        data: &'a [u8],
        _filename: &'a str,
    ) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        Box::pin(async move {
            if data.starts_with(b"%PDF") {
                return Ok((
                    MimeType::try_from("application/pdf").unwrap(),
                    "PDF document".to_string(),
                ));
            }
            if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                return Ok((
                    MimeType::try_from("image/png").unwrap(),
                    "PNG image data".to_string(),
                ));
            }
            Ok((
                MimeType::try_from("application/octet-stream").unwrap(),
                "data".to_string(),
            ))
        })
    }

    fn analyze_file<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        Box::pin(async move {
            let data = std::fs::read(path).map_err(|e| MagicError::FileNotFound(e.to_string()))?;
            if data.starts_with(b"%PDF") {
                return Ok((
                    MimeType::try_from("application/pdf").unwrap(),
                    "PDF document".to_string(),
                ));
            }
            if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                return Ok((
                    MimeType::try_from("image/png").unwrap(),
                    "PNG image data".to_string(),
                ));
            }
            Ok((
                MimeType::try_from("application/octet-stream").unwrap(),
                "data".to_string(),
            ))
        })
    }
}
