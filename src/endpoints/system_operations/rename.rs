use std::path::Path;
use actix_web::{post, web, HttpResponse, Responder};
use actix_web::http::StatusCode;
use crate::app_config::AppConfig;
use crate::models::authentication::auth_user::AuthenticatedUser;
use crate::models::system_operations::rename_item_request::RenameItemRequest;
use crate::services::file_structure::path_service::PathService;
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
    let path_service = PathService::new();
    let rename_service = RenameService::new(config.root_dir.as_ref().clone());
    let path_to_old_file = Path::new(config.root_dir.as_ref())
        .join(&username)
        .join(&path)
        .join(&old_name);
    let canonical = path_service.canonicalize_path(&path_to_old_file).await.unwrap();
    
    match path_service.check_if_entity_is_dir(&canonical).await {
        Ok(_) => {},
        Err((code, msg)) => 
            return HttpResponse::build(StatusCode::from_u16(code).unwrap()).body(msg),
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