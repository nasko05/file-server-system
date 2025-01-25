use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DownloadFileRequest {
    pub user_id: String,
    pub path_to_file: String,
    pub filename: String,
}