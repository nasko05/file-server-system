use crate::models::auth_user::AuthenticatedUser;
use crate::services::directory_service::{build_dir_tree, check_privilege_status};
use actix_web::{get, web, HttpResponse, Responder};
use std::path::PathBuf;

#[get("/directory/{dir_name}")]
async fn get_user_directory(
    path: web::Path<(String,)>,
    auth_user: AuthenticatedUser,
) -> impl Responder {
    println!("Got path {:?}", &path.0);
    let dir_name = &path.0;
    let user = auth_user.0.sub.as_str(); // 'claims' is the decoded JWT data

    if let Err(err) = check_privilege_status(dir_name, user).await {
        // Return the error as an HTTP response
        return HttpResponse::Forbidden().body(err.to_string());
    }

    // Build a path to: "root/<user>"
    let user_path = PathBuf::from("root").join(dir_name);

    // For safety, you might want to ensure the user can’t escape "root"
    // e.g., check that user does not contain ".." or something.

    match build_dir_tree(&user_path) {
        Ok(tree) => HttpResponse::Ok().json(tree), // Return JSON
        Err(err) => {
            // e.g., if directory doesn’t exist, or we lack permissions
            HttpResponse::NotFound().body(format!("Error reading directory: {}", err))
        }
    }
}