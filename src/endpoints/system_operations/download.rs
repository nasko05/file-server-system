use std::path::Path;
use crate::services::file_structure::file_service::FileService;
use actix_web::{post, web, HttpResponse, Responder};
use actix_web::http::StatusCode;
use log::info;
use crate::app_config::AppConfig;
use crate::models::authentication::auth_user::AuthenticatedUser;
use crate::models::system_operations::download_file_request::DownloadEntityRequest;
use crate::services::file_structure::directory_service::DirectoryService;

#[post("/download")]
pub async fn download_file_from_user_directory(
    payload: web::Json<DownloadEntityRequest>,
    authenticated_user: AuthenticatedUser,
    config: web::Data<AppConfig>,
) -> impl Responder {
    let username = authenticated_user.0.sub;
    let path = payload.path.as_str();
    let filename = payload.name.as_str();
    let file_service = FileService::new(
        config.root_dir.as_ref().clone(),
        config.directory_lock_manager.clone()
    );

    match file_service.read_file_from_any_directory(&username, path, filename).await {
        Ok((content, decoded_filename)) => {
            info!("Successfully downloaded: {}", decoded_filename);

            HttpResponse::Ok()
            .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", decoded_filename)))
            .body(content)
        },
        Err((code, msg)) => HttpResponse::build(StatusCode::from_u16(code).unwrap()).body(msg)
    }
}

#[post("/download/directory")]
pub async fn download_directory_from_user_directory(
    payload: web::Json<DownloadEntityRequest>,
    authenticated_user: AuthenticatedUser,
    config: web::Data<AppConfig>,
) -> impl Responder {
    let path = &payload.path;
    let name = &payload.name;
    let username = &authenticated_user.0.sub;
    let root = config.root_dir.as_ref();
    
    let dir_path = Path::new(root)
        .join(username)
        .join(path)
        .join(name);
    
    let directory_service = DirectoryService::new(
        root.clone(),
        config.directory_lock_manager.clone()
    );
    
    match directory_service.download_directory_streamed(dir_path).await {
        Ok(data) => HttpResponse::Ok().content_type("application/zip").body(data),
        Err((code, msg)) => HttpResponse::build(StatusCode::from_u16(code).unwrap()).body(msg)
    }
}