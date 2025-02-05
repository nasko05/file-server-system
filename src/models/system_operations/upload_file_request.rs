use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadFileRequest {
    pub file: Option<(String, Vec<u8>)>,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadRequestData {
    /// The final, sanitized filename.
    pub filename: Option<String>,
    /// The full file bytes read from the request.
    pub file_bytes: Option<Vec<u8>>,
    /// The path (directory) where the file should be stored.
    pub path: Option<String>,
}