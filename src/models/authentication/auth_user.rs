// src/auth.rs

use crate::services::authentication::authentication_service::Claims;
use actix_web::{
    dev::Payload,
    http::header::HeaderValue,
    Error,
    FromRequest,
    HttpRequest,
};
use futures::future::{ready, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;

/// Wrapper for claims extracted from a valid token.
pub struct AuthenticatedUser(pub Claims);

// If you want custom error types, you can define them here, but let's keep it simple:
impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // 1) Extract the "Authorization" header
        let token = match req.headers().get("Authorization") {
            Some(hv) => hv,
            None => {
                return ready(Err(actix_web::error::ErrorUnauthorized(
                    "Missing Authorization header",
                )));
            }
        };

        // 2) Parse "Bearer <token>"
        let token_str = match parse_bearer_token(token) {
            Ok(t) => t,
            Err(e) => {
                return ready(Err(actix_web::error::ErrorUnauthorized(e)));
            }
        };

        // 3) Decode + validate JWT.
        // You should store/use a real secret or public key for production.
        let secret = env::var("JWT_TOKEN_SECRET").expect("JWT_TOKEN_SECRET must be set");
        let validation = Validation::default();

        match decode::<Claims>(&token_str, &DecodingKey::from_secret(secret.as_ref()), &validation) {
            Ok(decoded) => {
                let claims = decoded.claims;
                ready(Ok(AuthenticatedUser(claims)))
            }
            Err(err) => {
                ready(Err(actix_web::error::ErrorUnauthorized(
                    format!("Invalid token: {}", err),
                )))
            }
        }
    }
}

fn parse_bearer_token(hv: &HeaderValue) -> Result<String, String> {
    let value = hv
        .to_str()
        .map_err(|_| "Invalid Authorization header".to_string())?;
    if !value.starts_with("Bearer ") {
        return Err("Expected 'Bearer <token>'".to_string());
    }
    let token = value.trim_start_matches("Bearer ").trim().to_string();
    if token.is_empty() {
        return Err("No token after 'Bearer '".to_string());
    }
    Ok(token)
}