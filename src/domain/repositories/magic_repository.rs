use std::path::Path;
use futures_util::future::BoxFuture;
use crate::domain::errors::MagicError;
use crate::domain::value_objects::mime_type::MimeType;

pub trait MagicRepository: Send + Sync {
    fn analyze_buffer<'a>(&'a self, data: &'a [u8], filename: &'a str) -> BoxFuture<'a, Result<(MimeType, String), MagicError>>;
    fn analyze_file<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, Result<(MimeType, String), MagicError>>;
}
