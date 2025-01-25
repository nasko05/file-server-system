use std::env;
use crate::{ROOT_DIR};
use actix_multipart::{Field};
use futures_util::TryStreamExt;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use crate::services::file_structure::directory_service::to_full_path;

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| *c != '/' && *c != '\\')
        .collect()
}

pub(crate) async fn save_file_to_root_directory(
    abs_path: &str,
    field: &mut Field,
) -> Result<String, String> {

    // Create the file asynchronously
    let mut file = match tokio::fs::File::create(abs_path).await {
        Ok(f) => f,
        Err(e) => {
            return Err(format!(
                "Error creating file at {}: {}",
                abs_path, e
            ));
        }
    };

    // Stream the file chunks directly to disk
    while let Ok(Some(chunk)) = field.try_next().await {
        if let Err(e) = file.write_all(&chunk).await {
            return Err(format!("Error writing to file {}: {}", abs_path, e));
        }
    }

    println!("Successfully saved file to {}", abs_path);

    Ok("Successfully saved file!".to_string())
}
pub(crate) async fn read_file_from_any_directory(
    user_name: &str,
    path: &str,
    filename: &str
) -> Result<(Vec<u8>, String), String> {

    // Construct the full file path
    let full_path = Path::new(ROOT_DIR)
        .join(user_name)
        .join(path.trim_start_matches('/'))
        .join(filename);

    // Prevent directory traversal attacks
    if full_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err("Invalid file path: directory traversal detected.".parse().unwrap());
    }
    
    println!("Current path: {}", env::current_dir().unwrap().display());
    
    let abs_path = match to_full_path(full_path) {
        Ok(abs_path) => abs_path,
        Err(e) => return Err(e),
    };
    
    // Use tokio::fs::read for asynchronous file reading
    match tokio::fs::read(abs_path).await {
        Ok(contents) => Ok((contents, filename.into())),
        Err(_) => Err(format!("File '{}' not found", filename)),
    }
}
