use std::path::Path;

pub struct RenameService {
    root_dir: String
}

impl RenameService {
    
    pub fn new(root_dir: String) -> Self {
        Self { root_dir }
    }
    
    pub async fn rename_directory(
        &self, 
        username: &String, 
        path: &String, 
        old_name: &String, 
        new_name: &String
    ) -> Result<String, (u16, String)>{
        match tokio::fs::rename(
            Path::new(&self.root_dir).join(username).join(path).join(old_name),
            Path::new(&self.root_dir).join(username).join(path).join(new_name)
        ).await {
            Ok(_) => Ok("Successfully renamed".parse().unwrap()),
            Err(e) => Err((400, format!("Error: {}", e))),
        }
    }
}