use actix_web::{post, web, HttpResponse, Responder};
use actix_web::http::StatusCode;
use log::{debug, error};
use crate::app_config::AppConfig;
use crate::models::authentication::auth_user::AuthenticatedUser;
use crate::models::system_operations::delete_file_request::DeleteEntityRequest;
use crate::services::file_structure::delete_service::DeleteService;

#[post("/directory/delete")]
pub async fn delete_user_directory(
    payload: web::Json<DeleteEntityRequest>,
    authenticated_user: AuthenticatedUser,
    config: web::Data<AppConfig>,
) -> impl Responder {
    debug!("Received payload: {:?}\n On Route /api/directory/delete", payload);
    let username = authenticated_user.0.sub;
    let dir_name = &payload.name;
    let path = &payload.path;

    let delete_service = DeleteService::new(config.root_dir.as_ref().clone());

    match delete_service.delete_directory(&username, path, dir_name).await {
        Ok(msg) => {
            debug!("Deleted directory {} successfully", dir_name);
            HttpResponse::Ok().body(msg)
        },
        Err((code, e)) => {
            error!("Failed to delete {}\nOn path: *{}*", dir_name, path);
            parse_status_code(code, e)
        }
    }
}

#[post("/file/delete")]
pub async fn delete_file(
    payload: web::Json<DeleteEntityRequest>,
    authenticated_user: AuthenticatedUser,
    config: web::Data<AppConfig>,
) -> impl Responder {
    debug!("Received payload:{:?}\n On Route /api/file/delete", payload);
    let username = authenticated_user.0.sub;
    let filename = &payload.name;
    let path = &payload.path;

    let delete_service = DeleteService::new(config.root_dir.as_ref().clone());

    match delete_service.delete_file(&username, path, filename).await {
        Ok(msg) => {
            debug!("Successfully delete file: {}", filename);
            HttpResponse::Ok().body(msg)
        },
        Err((code, e)) => {
            error!("Could not delete file: {}\nOn path: *{}*", filename, path);
            parse_status_code(code, e)
        }
    }
}

fn parse_status_code(code: u16, e: String) -> HttpResponse {
    let status_code = match StatusCode::from_u16(code) {
        Ok(c) => c,
        Err(err) => {
            error!("Could not parse return code {}", code);
            return HttpResponse::InternalServerError().body(format!("Could not parse return code {}. Error: {}", code, err));
        }
    };
    
    HttpResponse::build(status_code).body(e)
}