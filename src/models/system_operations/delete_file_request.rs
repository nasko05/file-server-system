use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DeleteEntityRequest {
    pub username: String,
    pub path: String,
    pub name: String,
}