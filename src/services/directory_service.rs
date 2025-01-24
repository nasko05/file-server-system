use std::{fs, io};
use std::path::Path;
use crate::models::directory_tree::DirTree;

pub fn build_dir_tree(path: &Path) -> io::Result<DirTree> {
    let name = path
        .file_name()
        .map(|os| os.to_string_lossy().into_owned())
        .unwrap_or_else(|| "".to_string());

    let mut files = Vec::new();
    let mut dirs = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            dirs.push(build_dir_tree(&entry_path)?);
        } else if metadata.is_file() {
            if let Some(fname) = entry_path.file_name() {
                files.push(fname.to_string_lossy().to_string());
            }
        }
    }

    Ok(DirTree {
        name,
        files,
        dirs,
    })
}