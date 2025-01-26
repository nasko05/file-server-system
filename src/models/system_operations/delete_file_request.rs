use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DeleteEntityRequest {
    pub path: String,
    pub name: String,
}