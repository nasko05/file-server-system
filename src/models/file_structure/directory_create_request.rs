use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryCreateRequest {
    pub path: String,
    pub name: String
}