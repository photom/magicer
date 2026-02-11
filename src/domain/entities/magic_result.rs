use crate::domain::value_objects::request_id::RequestId;
use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::domain::value_objects::mime_type::MimeType;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MagicResult {
    request_id: RequestId,
    filename: WindowsCompatibleFilename,
    mime_type: MimeType,
    description: String,
}

impl MagicResult {
    pub fn new(
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        mime_type: MimeType,
        description: String,
    ) -> Self {
        Self {
            request_id,
            filename,
            mime_type,
            description,
        }
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
}
