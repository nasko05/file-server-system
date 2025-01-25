use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DownloadFileRequest {
    pub username: String,
    pub path: String,
    pub filename: String,
}