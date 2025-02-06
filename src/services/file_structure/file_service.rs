use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::services::file_structure::path_service::PathService;
use crate::services::locking::directory_locking_manager::DirectoryLockManager;

pub struct FileService {
    root_dir: String,
    directory_lock_manager: DirectoryLockManager
}

impl FileService {
    pub fn new(root_dir: String, directory_lock_manager: DirectoryLockManager) -> Self {
        Self { root_dir, directory_lock_manager }
    }

    pub fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .filter(|c| *c != '/' && *c != '\\')
            .collect()
    }

    pub(crate) async fn save_file_bytes_to_root_directory(
        &self,
        abs_path: &PathBuf,
        file_bytes: &[u8],
    ) -> Result<String, (u16, String)> {
        
        // Create (or overwrite) the file asynchronously
        let mut file = match File::create(&abs_path).await {
            Ok(f) => f,
            Err(e) => {
                return Err((500, format!(
                    "Error creating file at {:?}: {}",
                    abs_path, e
                )));
            }
        };

        let lock_arc = self.directory_lock_manager.lock_for_path(abs_path.clone()).await;
        let _guard = lock_arc.lock().await;
        // Write the entire byte slice to the file
        if let Err(e) = file.write_all(file_bytes).await {
            return Err((500, format!("Error writing to file {:?}: {}", abs_path, e)));
        }

        println!("Successfully saved file to {:?}", abs_path);

        Ok("Successfully saved file!".to_string())
    }

    pub(crate) async fn read_file_from_any_directory(
        &self,
        user_name: &str,
        path: &str,
        filename: &str
    ) -> Result<(Vec<u8>, String), (u16, String)> {
        let path_service = PathService::new();
        // Construct the full file path
        let canonical = match path_service.canonicalize_path(&Path::new(&self.root_dir)
            .join(user_name)
            .join(path.trim_start_matches('/'))
            .join(filename)).await {
            Ok(path_buf) => path_buf,
            Err((code, msg)) => return Err((code, msg))
        };

        // Prevent directory traversal attacks
        if canonical.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err((400, "Invalid file path: directory traversal detected.".parse().unwrap()));
        }

        let lock_arc = self.directory_lock_manager.lock_for_path(canonical.clone()).await;
        let _guard = lock_arc.lock().await;
        // Use tokio::fs::read for asynchronous file reading
        match tokio::fs::read(&canonical).await {
            Ok(contents) => Ok((contents, filename.into())),
            Err(_) => Err((404, format!("File '{}' not found", filename))),
        }
    }
}
