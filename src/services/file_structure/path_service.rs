use std::path::PathBuf;
use log::error;
use tokio::fs;

pub struct PathService;

impl PathService {
    pub fn new() -> Self {
        Self { }
    }
    
    pub async fn canonicalize_path(&self, path: &PathBuf) -> Result<PathBuf, (u16, String)>{
        match fs::canonicalize(&path).await {
            Ok(path) => Ok(path),
            Err(err) => {
                Err((404, format!("Invalid directory/file '{}': {}", path.display(), err)))
            }
        }
    }

    pub async fn check_if_entity_is_dir(&self, canonical: &PathBuf) -> Result<(), (u16, String)> {
        // Check if the directory exists and delete it
        match tokio::fs::metadata(&canonical).await {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    error!("{} is not a directory", canonical.to_str().unwrap());
                    Err((400, format!("'{}' is not a directory.", canonical.to_str().unwrap())))
                } else {
                    Ok(())
                }
            }
            Err(_) => {
                error!("{} is not found", canonical.to_str().unwrap());
                Err((404, format!("Directory '{}' not found.", canonical.to_str().unwrap())))
            },
        }
    }

    pub async fn check_if_entity_is_file(&self, canonical: &PathBuf) -> Result<(), (u16, String)> {
        match tokio::fs::metadata(&canonical).await {
            Ok(metadata) => {
                if !metadata.is_file() {
                    error!("{} is not a directory", canonical.to_str().unwrap());
                    Err((400, format!("'{}' is not a directory.", canonical.to_str().unwrap())))
                } else {
                    Ok(())
                }
            }
            Err(_) => {
                error!("{} is not found", canonical.to_str().unwrap());
                Err((404, format!("Directory '{}' not found.", canonical.to_str().unwrap())))
            },
        }
    }
}