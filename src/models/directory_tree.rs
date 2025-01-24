use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DirTree {
    pub name: String,
    pub files: Vec<String>,
    pub dirs: Vec<DirTree>,
}
