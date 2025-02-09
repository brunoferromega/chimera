use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Json, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub email: String,
}

impl Claims {
    pub fn new(exp: usize, iat: usize, email: String) -> Self {
        Self { exp, iat, email }
    }
}

#[derive(Deserialize)]
pub struct SignInData {
    pub email: String,
    pub password: String,
}

#[derive(Clone)]
struct CurrentUser {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password_hash: String,
}

pub async fn sign_in(Json(user_data): Json<SignInData>, _state: Arc<PgPool>) -> Response {
    let user = match user_finded(&user_data.email) {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED).into_response(),
    };

    if !verify(&user_data.password, &user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR).into_response())
        .unwrap()
    {
        return (StatusCode::UNAUTHORIZED).into_response();
    }

    let token = encode_jwt(user.email)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap();

    (StatusCode::OK, Json(token)).into_response()
}

fn user_finded(email: &str) -> Option<CurrentUser> {
    if email == "fakecrypt@mail.com" {
        let ph = hash("rusty", DEFAULT_COST).unwrap();
        println!("{}", &ph);

        Some(CurrentUser {
            email: email.to_string(),
            first_name: "Thanatos".to_string(),
            last_name: "Niet".to_string(),
            password_hash: ph,
        })
    } else {
        None
    }
}

fn encode_jwt(email: String) -> Result<String, StatusCode> {
    let secret = "randomStringTypicallyFromEnv".to_string();
    let now = Utc::now();
    let expire: chrono::TimeDelta = Duration::hours(24);
    let exp: usize = (now + expire).timestamp() as usize;
    let iat: usize = now.timestamp() as usize;
    let claim = Claims::new(exp, iat, email);

    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
