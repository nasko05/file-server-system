use actix_web::{
    App, HttpServer
};

static ROOT_DIR: &str = "./root";

mod endpoints;
mod utilities;

use crate::endpoints::download::download_file;
use crate::endpoints::upload::upload_file;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Ensure the upload directory exists
    std::fs::create_dir_all(ROOT_DIR)?;

    println!("Server running on http://0.0.0.0:8080");

    HttpServer::new(|| {
        App::new()
            .service(download_file)
            .service(upload_file)
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}