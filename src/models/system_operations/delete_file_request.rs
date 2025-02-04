use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteEntityRequest {
    pub path: String,
    pub name: String,
}