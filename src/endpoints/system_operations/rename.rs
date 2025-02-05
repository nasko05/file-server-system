use actix_web::{post, web, HttpResponse, Responder};
use actix_web::http::StatusCode;
use crate::app_config::AppConfig;
use crate::models::authentication::auth_user::AuthenticatedUser;
use crate::models::system_operations::rename_item_request::RenameItemRequest;
use crate::services::file_structure::directory_service::DirectoryService;
use crate::services::file_structure::rename_service::RenameService;

#[post("/directory/rename")]
pub async fn rename_directory(
    req: web::Json<RenameItemRequest>,
    authenticated_user: AuthenticatedUser,
    config: web::Data<AppConfig>
) -> impl Responder {
    let username = authenticated_user.0.sub;
    let path = &req.path;
    let old_name = &req.old_name;
    let new_name = &req.new_name;
    let directory_service = DirectoryService::new(config.root_dir.as_ref().clone());
    let rename_service = RenameService::new(config.root_dir.as_ref().clone());
    
    match directory_service.check_if_directory_exists(path, &username, old_name).await {
        Ok(a) => { 
            if a != "dir" && a != "file"{
                return HttpResponse::BadRequest().body("You requested to rename a directory or file!")
            }
        },
        Err(_) => return HttpResponse::NotFound().body("File not found"),
    }
    
    
    match rename_service.rename_directory(
        &username,
        path,
        old_name,
        new_name
    ).await {
        Ok(msg) => HttpResponse::Ok().body(msg),
        Err((code, msg)) => HttpResponse::build(StatusCode::try_from(code).unwrap()).body(msg)
    }
}