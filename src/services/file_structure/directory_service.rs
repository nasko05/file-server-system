use crate::dao::login_verification::check_privileges;
use std::path::Path;
use std::{fs, io};
use crate::models::file_structure::directory_tree::DirTree;

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