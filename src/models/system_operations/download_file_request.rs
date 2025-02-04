use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadFileRequest {
    pub path: String,
    pub filename: String,
}