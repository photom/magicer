use crate::domain::errors::MagicError;
use crate::domain::value_objects::mime_type::MimeType;
use futures_util::future::BoxFuture;

pub trait MagicRepository: Send + Sync {
    fn analyze_buffer<'a>(
        &'a self,
        data: &'a [u8],
        filename: &'a str,
    ) -> BoxFuture<'a, Result<(MimeType, String), MagicError>>;
}
