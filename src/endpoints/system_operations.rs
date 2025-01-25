use std::path::PathBuf;
use actix_web::{error, get, web, HttpResponse, Responder};
use crate::dao::login_verification::check_privileges;
use crate::models::auth_user::AuthenticatedUser;
use crate::services::directory_service::build_dir_tree;

#[get("/directory/{dir_name}")]
async fn get_user_directory(
    path: web::Path<(String,)>,
    auth_user: AuthenticatedUser,
) -> impl Responder {
    println!("Got path {:?}", &path.0);
    let dir_name = &path.0;
    let claims = auth_user.0; // 'claims' is the decoded JWT data

    let to_be_accessed = check_privileges(dir_name).await.expect(format!("The role {} does not exist", dir_name).as_str());
    let actual_privileges = check_privileges(claims.sub.as_str()).await.expect(format!("The role {} does not exist", claims.sub.as_str()).as_str());

    // Compare the route param to the user's token role
    if actual_privileges < to_be_accessed {
        // If they don't match, return 403
        return Err(error::ErrorForbidden(format!(
            "Your token role is '{}', but you tried to access '{}'",
            claims.sub, dir_name
        )));
    }

    // Build a path to: "root/<user>"
    let user_path = PathBuf::from("root").join(dir_name);

    // For safety, you might want to ensure the user can’t escape "root"
    // e.g., check that user does not contain ".." or something.

    match build_dir_tree(&user_path) {
        Ok(tree) => Ok(HttpResponse::Ok().json(tree)), // Return JSON
        Err(err) => {
            // e.g., if directory doesn’t exist, or we lack permissions
            Ok(HttpResponse::NotFound().body(format!("Error reading directory: {}", err)))
        }
    }
}