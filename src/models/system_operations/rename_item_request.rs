use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RenameItemRequest {
    pub path: String,
    pub old_name: String,
    pub new_name: String
}