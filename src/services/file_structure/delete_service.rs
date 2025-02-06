use std::path::Path;
use std::sync::Arc;
use crate::services::file_structure::path_service::PathService;
use crate::services::locking::directory_locking_manager::DirectoryLockManager;

pub struct DeleteService {
    root_dir: String,
    directory_lock_manager: DirectoryLockManager
}

impl DeleteService {
    pub fn new(root_dir: String, directory_lock_manager: DirectoryLockManager) -> Self {
        Self { 
            root_dir, directory_lock_manager
        }
    }
    
    pub async fn delete_directory(
        &self,
        username: &String, 
        path: &String, 
        dir_name: &String
    ) -> Result<String, (u16, String)> {
        // Construct the path to the directory
        let dir_path = Path::new(&self.root_dir)
            .join(username)
            .join(path)
            .join(dir_name);

        let path_service = PathService::new();
        let canonical = match path_service.canonicalize_path(&dir_path).await {
            Ok(path) => path,
            Err((code, msg)) => return Err((code, msg))
        };
        
        match path_service.check_if_entity_is_dir(&canonical).await {
            Ok(_) => {},
            Err((code, msg)) => return Err((code, msg))
        }
        
        let lock_arc = self.directory_lock_manager.lock_for_path(canonical.clone()).await;
        let _guard = lock_arc.lock().await;
        
        let remove_result = tokio::fs::remove_dir(&canonical).await;
        match remove_result {
            Ok(_) => {
                {
                    let mut map = self.directory_lock_manager.locks.lock().await;
                    // If no one else is using this lock, remove it
                    if Arc::strong_count(&lock_arc) == 1 {
                        map.remove(&canonical);
                    }
                }
                Ok(format!("Directory '{}' deleted successfully.", dir_name))
            },
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    Err((404, format!("Directory '{}' not found.", dir_name)))
                } else if err.kind() == std::io::ErrorKind::Other {
                    Err((400, format!("'{}' is not a directory or is inaccessible.", dir_name)))
                } else {
                    Err((500, format!("Failed to delete directory '{}': {:?}", dir_name, err)))
                }
            }
        }
    }
    
    pub async fn delete_file(
        &self,
        username: &String, 
        path: &String, 
        filename: &String
    ) -> Result<String, (u16, String)> {
        // Construct the path to the file
        let dir_path = Path::new(&self.root_dir)
            .join(username)
            .join(path)
            .join(filename);

        let path_service = PathService::new();
        let canonical = match path_service.canonicalize_path(&dir_path).await {
            Ok(path) => path,
            Err((code, msg)) => return Err((code, msg))
        };

        match path_service.check_if_entity_is_file(&canonical).await {
            Ok(_) => {},
            Err((code, msg)) => return Err((code, msg))
        }

        let lock_arc = self.directory_lock_manager.lock_for_path(canonical.clone()).await;
        let _guard = lock_arc.lock().await;

        let remove_result = tokio::fs::remove_file(&canonical).await;
        match remove_result {
            Ok(_) => {
                {
                    let mut map = self.directory_lock_manager.locks.lock().await;
                    // If no one else is using this lock, remove it
                    if Arc::strong_count(&lock_arc) == 1 {
                        map.remove(&canonical);
                    }
                }
                Ok(format!("File '{}' deleted successfully.", filename))
            },
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    Err((404, format!("File '{}' not found.", filename)))
                } else if err.kind() == std::io::ErrorKind::Other {
                    Err((400, format!("'{}' is not a file or is inaccessible.", filename)))
                } else {
                    Err((500, format!("Failed to delete file '{}': {:?}", filename, err)))
                }
            }
        }
    }
}