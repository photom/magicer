use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::domain::value_objects::mime_type::MimeType;
use crate::domain::value_objects::request_id::RequestId;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MagicResult {
    id: Uuid,
    request_id: RequestId,
    filename: WindowsCompatibleFilename,
    mime_type: MimeType,
    description: String,
    encoding: Option<String>,
    analyzed_at: DateTime<Utc>,
}

impl MagicResult {
    pub fn new(
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        mime_type: MimeType,
        description: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            request_id,
            filename,
            mime_type,
            description,
            encoding: None,
            analyzed_at: Utc::now(),
        }
    }

    pub fn with_encoding(mut self, encoding: Option<String>) -> Self {
        self.encoding = encoding;
        self
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    pub fn filename(&self) -> &WindowsCompatibleFilename {
        &self.filename
    }

    pub fn mime_type(&self) -> &MimeType {
        &self.mime_type
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }

    pub fn analyzed_at(&self) -> DateTime<Utc> {
        self.analyzed_at
    }
}

impl PartialEq for MagicResult {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for MagicResult {}
