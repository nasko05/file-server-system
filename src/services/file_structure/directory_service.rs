use crate::dao::login_verification::check_privileges;
use std::path::{Path, PathBuf};
use std::{fs, io};
use std::future::Future;
use crate::models::file_structure::directory_tree::DirTree;
use crate::ROOT_DIR;

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

pub async fn check_privilege_status(dir_name: &str, user_name: &str) -> Result<(), String> {
    let to_be_accessed = check_privileges(dir_name).await.expect(format!("The role {} does not exist", dir_name).as_str());
    let actual_privileges = check_privileges(user_name).await.expect(format!("The role {} does not exist", user_name).as_str());

    // Compare the route param to the user's token role
    if actual_privileges < to_be_accessed {
        // If they don't match, return 403
        return Err(format!(
            "Your token role is '{}', but you tried to access '{}'",
            user_name, dir_name
        ));
    }

    Ok(())
}

pub fn to_full_path(relative_path: PathBuf) -> Result<String, String> {
    let path = Path::new(&relative_path);
    match path.canonicalize() {
        Ok(absolute_path) => Ok(absolute_path.to_string_lossy().to_string()),
        Err(e) => Err(format!("Failed to convert to absolute path: {:?}", e)),
    }
}

pub async fn check_if_directory_exists(directory_name: &str, username: &str, name: &str) -> Result<String, String> {
    match tokio::fs::metadata(
        Path::new(ROOT_DIR)
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