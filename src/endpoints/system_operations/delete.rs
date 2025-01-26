use actix_web::{post, web, HttpResponse, Responder};
use std::path::{Path};
use crate::models::authentication::auth_user::AuthenticatedUser;
use crate::models::system_operations::delete_file_request::DeleteEntityRequest;
use crate::ROOT_DIR;



#[post("/directory/delete")]
pub async fn delete_user_directory(
    payload: web::Json<DeleteEntityRequest>,
    authenticated_user: AuthenticatedUser
) -> impl Responder {
    let username = authenticated_user.0.sub;
    let dir_name = payload.name.as_str();
    let path = payload.path.as_str();

    // Construct the path to the directory
    let dir_path = Path::new(ROOT_DIR).join(username).join(path).join(dir_name);

    // Check if the directory exists and delete it
    match tokio::fs::metadata(&dir_path).await {
        Ok(metadata) => {
            if metadata.is_dir() {
                match tokio::fs::remove_dir_all(&dir_path).await {
                    Ok(_) => HttpResponse::Ok().body(format!("Directory '{}' deleted successfully.", dir_name)),
                    Err(err) => HttpResponse::InternalServerError().body(format!("Failed to delete directory '{}': {:?}", dir_name, err)),
                }
            } else {
                HttpResponse::BadRequest().body(format!("'{}' is not a directory.", dir_name))
            }
        }
        Err(_) => HttpResponse::NotFound().body(format!("Directory '{}' not found.", dir_name)),
    }
}

#[post("/file/delete")]
pub async fn delete_file(
    payload: web::Json<DeleteEntityRequest>,
    authenticated_user: AuthenticatedUser
) -> impl Responder {
    let username = authenticated_user.0.sub;
    let filename = payload.name.as_str();
    let path = payload.path.as_str();

    // Construct the path to the file
    let file_path = Path::new(ROOT_DIR).join(username).join(path).join(&filename);

    // Check if the file exists and delete it
    match tokio::fs::metadata(&file_path).await {
        Ok(metadata) => {
            if metadata.is_file() {
                match tokio::fs::remove_file(&file_path).await {
                    Ok(_) => HttpResponse::Ok().body(format!("File '{}' deleted successfully.", filename)),
                    Err(err) => HttpResponse::InternalServerError().body(format!("Failed to delete file '{}': {:?}", filename, err)),
                }
            } else {
                HttpResponse::BadRequest().body(format!("'{}' is not a file.", filename))
            }
        }
        Err(_) => HttpResponse::NotFound().body(format!("File '{}' not found.", filename)),
    }
}