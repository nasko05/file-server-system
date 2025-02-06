use std::path::{Path};
use actix_web::{post, web, HttpResponse, Responder};
use crate::app_config::AppConfig;
use crate::models::authentication::auth_user::AuthenticatedUser;
use crate::models::file_structure::file_structure_request::FileStructureRequest;
use crate::services::file_structure::directory_service::DirectoryService;

#[post("/structure")]
async fn get_user_directory(
    payload: web::Json<FileStructureRequest>,
    auth_user: AuthenticatedUser,
    config: web::Data<AppConfig>
) -> impl Responder {
    let dir_name = Path::new(&payload.path);
    let user = auth_user.0.sub;

    let directory_service = DirectoryService::new(
        config.root_dir.as_ref().clone(),
        config.directory_lock_manager.clone()
    );

    match directory_service.build_dir_tree(&user, dir_name) {
        Ok(tree) => HttpResponse::Ok().json(tree),
        Err(err) => {
            HttpResponse::NotFound().body(format!("Error reading directory: {}", err))
        }
    }
}