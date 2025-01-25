use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DeleteEntityRequest {
    pub username: String,
    pub name: String,
    pub path: String
}