use std::path::Path;
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::authentication::auth_user::AuthenticatedUser;
use crate::models::system_operations::rename_item_request::RenameItemRequest;
use crate::ROOT_DIR;
use crate::services::file_structure::directory_service::DirectoryService;

#[post("/directory/rename")]
pub async fn rename_directory(
    req: web::Json<RenameItemRequest>,
    authenticated_user: AuthenticatedUser
) -> impl Responder {
    let username = authenticated_user.0.sub;
    let path = req.path.as_str();
    let old_name = req.old_name.as_str();
    let new_name = req.new_name.as_str();
    let directory_service = DirectoryService::new(ROOT_DIR.into());
    
    match directory_service.check_if_directory_exists(path, &username, old_name).await {
        Ok(a) => { 
            if a != "dir" && a != "file"{
                return HttpResponse::BadRequest().body("You requested to rename a directory or file!")
            }
        },
        Err(_) => return HttpResponse::NotFound().body("File not found"),
    }
    
    
    match tokio::fs::rename(
        Path::new(ROOT_DIR).join(&username).join(path).join(old_name),
        Path::new(ROOT_DIR).join(&username).join(path).join(new_name)
    ).await {
        Ok(_) => HttpResponse::Ok().body("Successfully renamed"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}