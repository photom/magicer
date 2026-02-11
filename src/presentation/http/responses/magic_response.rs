use serde::Serialize;
use chrono::{DateTime, Utc};
use crate::domain::entities::magic_result::MagicResult;

#[derive(Serialize)]
pub struct MagicResponse {
    pub request_id: String,
    pub filename: String,
    pub result: MagicAnalysisResult,
    pub encoding: Option<String>,
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct MagicAnalysisResult {
    pub mime_type: String,
    pub description: String,
}

impl From<MagicResult> for MagicResponse {
    fn from(result: MagicResult) -> Self {
        Self {
            request_id: result.request_id().as_str().to_string(),
            filename: result.filename().as_str().to_string(),
            result: MagicAnalysisResult {
                mime_type: result.mime_type().as_str().to_string(),
                description: result.description().to_string(),
            },
            encoding: result.encoding().map(|s| s.to_string()),
            analyzed_at: result.analyzed_at(),
        }
    }
}
