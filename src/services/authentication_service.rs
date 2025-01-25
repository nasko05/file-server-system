use std::env;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use once_cell::sync::Lazy;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // User ID
    pub exp: usize,   // Expiration timestamp
}

static SECRET_KEY: Lazy<Vec<u8>> = Lazy::new(|| {
    env::var("JWT_TOKEN_SECRET")
        .expect("JWT_TOKEN_SECRET must be set")
        .into_bytes()
});
pub fn generate_jwt(user_id: String) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600; // Expires in 1 hour

    let claims = Claims {
        sub: user_id,
        exp: expiration as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(&SECRET_KEY))
}

pub fn validate_jwt_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&SECRET_KEY),
        &Validation::new(Algorithm::HS256),
    )?;
    Ok(token_data.claims)
}