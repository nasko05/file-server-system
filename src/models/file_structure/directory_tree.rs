use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DirTree {
    pub name: String,
    pub files: Vec<String>,
    pub dirs: Vec<DirTree>,
}
