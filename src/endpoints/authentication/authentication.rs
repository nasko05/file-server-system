use crate::dao::login_verification::verify_user_credentials;
use crate::services::authentication::authentication_service::Claims;
use crate::services::authentication::authentication_service::generate_jwt;
use actix_web::{post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

#[post("/login")]
pub async fn login_handler(user_info: web::Json<UserLogin>) -> impl Responder {
    let user_id = match verify_user_credentials(
        user_info.username.as_str(), user_info.password.as_str()
    ).await {
        Ok(id) => id,
        Err(e) => return HttpResponse::NotFound().body(format!("{:?}", e))
    };
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