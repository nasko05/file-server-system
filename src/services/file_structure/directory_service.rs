use std::path::{Path, PathBuf};
use std::{fs, io};
use crate::models::file_structure::directory_tree::DirTree;

pub struct DirectoryService {
    root_dir: String
}

impl DirectoryService {
    
    pub fn new(root_dir: String) -> Self {
        Self {
            root_dir
        }
    }
    
    pub fn build_dir_tree(&self, path: &Path) -> io::Result<DirTree> {
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
                dirs.push(self.build_dir_tree(&entry_path)?);
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

    pub fn to_full_path(&self, relative_path: PathBuf) -> Result<String, String> {
        let path = Path::new(&relative_path);
        match path.canonicalize() {
            Ok(absolute_path) => Ok(absolute_path.to_string_lossy().to_string()),
            Err(e) => Err(format!("Failed to convert to absolute path: {:?}", e)),
        }
    }

    pub async fn check_if_directory_exists(
        &self, 
        directory_name: &str, 
        username: &str, 
        name: &str
    ) -> Result<String, String> {
        match tokio::fs::metadata(
            Path::new(&self.root_dir)
                .join(username)
                .join(directory_name)
                .join(name)
        ).await {
            Ok(metadata) => {
                if metadata.is_dir() {
                    Ok("dir".parse().unwrap())
                } else if metadata.is_file() {
                    Ok("file".parse().unwrap())
                } else {
                    Err(format!("The directory '{}' does not exist", directory_name))
                }
            },
            Err(_e) => Err(format!("Directory/File '{}' does not exist", directory_name)),
        }
    }
}