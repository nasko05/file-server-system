use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DownloadFileRequest {
    pub path: String,
    pub filename: String,
}