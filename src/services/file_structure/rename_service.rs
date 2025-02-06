use std::path::Path;
use log::error;
use crate::services::file_structure::path_service::PathService;

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
        let old_path = Path::new(&self.root_dir).join(username).join(path).join(old_name);
        let new_path = Path::new(&self.root_dir).join(username).join(path).join(new_name);
        
        let path_service = PathService::new();

        let canonical_old = path_service.canonicalize_path(&old_path)
            .await.expect("Could not canonicalize old path");
        let canonical_new = path_service.canonicalize_path(&new_path)
            .await.expect("Could not canonicalize new path");
        
        match tokio::fs::rename(
            &canonical_old,
            &canonical_new
        ).await {
            Ok(_) => Ok("Successfully renamed".parse().unwrap()),
            Err(e) => {
                error!("{}", e);
                Err((400, format!("Error: {}", e)))
            },
        }
    }
}