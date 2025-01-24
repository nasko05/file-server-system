use actix_web::{post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use crate::services::authentication_service::{generate_jwt};
use crate::services::authentication_service::Claims;

#[derive(Debug, Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

#[post("/login")]
pub async fn login_handler(user_info: web::Json<UserLogin>) -> impl Responder {
    // TODO: Normally, you'd validate `user_info.username` and `user_info.password` against a database.
    let user_id = "user_id_123".to_owned();
    match generate_jwt(user_id) {
        Ok(token) => HttpResponse::Ok().json(serde_json::json!({ "token": token })),
        Err(_) => HttpResponse::InternalServerError().body("Could not generate token"),
    }
}

pub async fn protected_resource_handler(req: HttpRequest) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        HttpResponse::Ok().json(serde_json::json!({
            "msg": "You have accessed a protected resource!",
            "user_id": claims.sub,
        }))
    } else {
        HttpResponse::Unauthorized().body("Unauthorized")
    }
}