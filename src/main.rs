use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use crate::endpoints::authentication::{login_handler, protected_resource_handler};

static ROOT_DIR: &str = "./root";

mod endpoints;
mod utilities;
mod services;
mod models;
mod dao;

use crate::endpoints::download::download_file_from_root_directory;
use crate::endpoints::download::download_file_from_user_directory;
use crate::endpoints::upload::upload_file_from_root_directory;
use crate::endpoints::upload::upload_file_from_user_directory;
use crate::endpoints::system_operations::get_user_directory;

use dotenv::dotenv;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Ensure the upload directory exists
    std::fs::create_dir_all(ROOT_DIR)?;
    dotenv().ok();

    println!("Server running on http://0.0.0.0:8080");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(login_handler)
            .service(
                web::scope("/api")
                    .wrap(models::auth_models::JwtAuth)
                    .service(web::resource("/protected").route(web::get().to(protected_resource_handler)))
                    .service(download_file_from_root_directory)
                    .service(download_file_from_user_directory)
                    .service(upload_file_from_root_directory)
                    .service(upload_file_from_user_directory)
                    .service(get_user_directory)
            )
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}