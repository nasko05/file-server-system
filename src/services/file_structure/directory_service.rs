use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};
use actix_web::HttpResponse;
use log::error;
use crate::models::file_structure::directory_tree::DirTree;
use zip::ZipWriter;
use zip::write::FileOptions;
use walkdir::WalkDir;

pub struct DirectoryService {
    root_dir: String
}

impl DirectoryService {
    
    pub fn new(root_dir: String) -> Self {
        Self {
            root_dir
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

    pub fn to_full_path(&self, relative_path: PathBuf) -> Result<String, String> {
        let path = Path::new(&relative_path);
        match path.canonicalize() {
            Ok(absolute_path) => Ok(absolute_path.to_string_lossy().to_string()),
            Err(e) => {
                error!("{}", e);
                Err(format!("Failed to convert to absolute path: {:?}", e))
            },
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
                    error!("{:?}", metadata);
                    Err(format!("Entity '{}' is neither a file nor directory", directory_name))
                }
            },
            Err(e) => {
                error!("{}", e);
                Err(format!("Directory/File '{}' does not exist", directory_name))
            },
        }
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

        match tokio::fs::create_dir(path).await {
            Ok(_) => Ok("Successfully created the dir!".into()),
            Err(e) => {
                error!("{}", e);
                Err((400, e.to_string()))
            }
        }
    }

    pub async fn download_directory_streamed(&self, dir_path: PathBuf) -> Result<Vec<u8>, String> {
        // 1. Create a temp file
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();

        {
            // 2. Build the ZIP using ZipWriter (which needs Write + Seek)
            let mut zip = ZipWriter::new(&mut temp_file);
            let options = FileOptions::default();

            for entry in WalkDir::new(dir_path.clone()) {
                let entry = entry.unwrap();
                let file_path = entry.path();
                if file_path.is_file() {
                    let relative_path = file_path.strip_prefix(dir_path.clone()).unwrap();
                    let name_in_zip = relative_path.to_string_lossy();

                    zip.start_file(name_in_zip, options).unwrap();
                    let bytes = fs::read(file_path).unwrap();
                    zip.write_all(&bytes).unwrap();
                }
            }
            zip.finish().unwrap();
        }

        // 3. Rewind the file so we can read it into the HTTP response
        temp_file.as_file_mut().seek(SeekFrom::Start(0)).unwrap();

        // 4. Stream or read the file contents in your response
        // (For large files, you might do a streaming approach. For simplicity, read to memory here.)
        let data = fs::read(temp_file.path()).unwrap();

        Ok(data)
    }
}