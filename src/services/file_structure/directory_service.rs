use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};
use log::error;
use crate::models::file_structure::directory_tree::DirTree;
use zip::ZipWriter;
use zip::write::FileOptions;
use walkdir::WalkDir;
use crate::services::file_structure::path_service::PathService;
use crate::services::locking::directory_locking_manager::DirectoryLockManager;

pub struct DirectoryService {
    root_dir: String,
    directory_lock_manager: DirectoryLockManager
}

impl DirectoryService {
    
    pub fn new(root_dir: String, directory_lock_manager: DirectoryLockManager) -> Self {
        Self {
            root_dir,
            directory_lock_manager
        }
    }

    pub fn build_dir_tree(&self, user: &String, path: &Path) -> io::Result<DirTree> {
        // Construct the full path from user + path
        let full_path = Path::new(&self.root_dir).join(user).join(path);

        // The "name" is the final component of `path`
        let mut name = path
            .file_name()
            .map(|os| os.to_string_lossy().into_owned())
            .unwrap_or_default();

        if name.is_empty() {
            name = user.clone();
        }
        let mut files = Vec::new();
        let mut dirs = Vec::new();

        for entry in fs::read_dir(&full_path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                // For subdirs, we extend `path` by the subdirectory name
                let sub_path = path.join(entry.file_name());
                dirs.push(self.build_dir_tree(user, &sub_path)?);
            } else if metadata.is_file() {
                if let Some(fname) = entry_path.file_name() {
                    files.push(fname.to_string_lossy().to_string());
                }
            }
        }

        Ok(DirTree { name, files, dirs })
    }

    pub async fn create_directory(
        &self,
        user: &String,
        path: &String,
        name: &String
    ) -> Result<String, (u16, String)> {
        let path = Path::new(&self.root_dir)
            .join(user)
            .join(path)
            .join(name);
        
        let path_service = PathService::new();
        let canonical = match path_service.canonicalize_path(&path).await
        {
            Ok(res) => res,
            Err((code, msg)) => return Err((code, msg))
        };

        match tokio::fs::create_dir(&canonical).await {
            Ok(_) => Ok("Successfully created the dir!".into()),
            Err(e) => {
                error!("{}", e);
                Err((400, e.to_string()))
            }
        }
    }

    pub async fn download_directory_streamed(&self, dir_path: PathBuf) -> Result<Vec<u8>, (u16, String)> {
        let path_service = PathService::new();
        let canonical = match path_service.canonicalize_path(&dir_path).await {
            Ok(res) => res,
            Err((code, msg)) => return Err((code, msg)) 
        };
        
        match path_service.check_if_entity_is_dir(&canonical).await {
            Ok(_) => {},
            Err((code, msg)) => return Err((code, msg))
        }

        let lock_arc = self.directory_lock_manager.lock_for_path(canonical.clone()).await;
        let _guard = lock_arc.lock().await;
        
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();

        {
            // 2. Build the ZIP using ZipWriter (which needs Write + Seek)
            let mut zip = ZipWriter::new(&mut temp_file);
            let options = FileOptions::default();

            for entry in WalkDir::new(&dir_path) {
                let entry = entry.unwrap();
                let file_path = entry.path();
                if file_path.is_file() {
                    let relative_path = file_path.strip_prefix(&dir_path).unwrap();
                    let name_in_zip = relative_path.to_string_lossy();

                    zip.start_file(name_in_zip, options).unwrap();
                    let bytes = fs::read(file_path).unwrap();
                    zip.write_all(&bytes).unwrap();
                }
            }
            zip.finish().unwrap();
        }

        temp_file.as_file_mut().seek(SeekFrom::Start(0)).unwrap();

        let data = fs::read(temp_file.path()).unwrap();

        Ok(data)
    }
}