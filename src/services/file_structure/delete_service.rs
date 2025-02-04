use std::path::Path;

pub struct DeleteService {
    root_dir: String
}

impl DeleteService {
    pub fn new(root_dir: String) -> Self {
        Self { root_dir }
    }
    
    pub async fn delete_directory(
        &self,
        username: &String, 
        path: &String, 
        dir_name: &String
    ) -> Result<String, (u16, String)> {
        // Construct the path to the directory
        let dir_path = Path::new(&self.root_dir).join(username).join(path).join(dir_name);

        // Check if the directory exists and delete it
        match tokio::fs::metadata(&dir_path).await {
            Ok(metadata) => {
                if metadata.is_dir() {
                    match tokio::fs::remove_dir_all(&dir_path).await {
                        Ok(_) => Ok(format!("Directory '{}' deleted successfully.", dir_name)),
                        Err(err) => Err((500, format!("Failed to delete directory '{}': {:?}", dir_name, err))),
                    }
                } else {
                    Err((400, format!("'{}' is not a directory.", dir_name)))
                }
            }
            Err(_) => Err((404, format!("Directory '{}' not found.", dir_name))),
        }
    }
    
    pub async fn delete_file(
        &self,
        username: &String, 
        path: &String, 
        filename: &String
    ) -> Result<String, (u16, String)> {
        // Construct the path to the file
        let full_path = Path::new(&self.root_dir).join(username).join(path).join(filename);
        let file_path = full_path.to_str().unwrap();

        // Check if the file exists and delete it
        match tokio::fs::metadata(file_path).await {
            Ok(metadata) => {
                if metadata.is_file() {
                    match tokio::fs::remove_file(file_path).await {
                        Ok(_) => Ok(format!("File '{}' deleted successfully.", filename)),
                        Err(err) => Err((500, format!("Failed to delete file '{}': {:?}", filename, err))),
                    }
                } else {
                    Err((400, format!("'{}' is not a file.", filename)))
                }
            }
            Err(_) => Err((404, format!("File '{}' not found.", filename))),
        }
    }
}