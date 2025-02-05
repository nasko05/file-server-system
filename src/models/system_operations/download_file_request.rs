use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadEntityRequest {
    pub path: String,
    pub name: String,
}