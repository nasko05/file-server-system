use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RenameItemRequest {
    pub path: String,
    pub old_name: String,
    pub new_name: String
}