use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
pub struct FileStructureRequest {
    pub path: String
}