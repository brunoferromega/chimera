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

pub async fn sign_in(Json(user_data): Json<SignInData>, state: Arc<PgPool>) -> Response {
    let _ = match user_finded(&user_data.email) {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED).into_response(),
    };

    (StatusCode::OK).into_response()
}

fn user_finded(email: &str) -> Option<CurrentUser> {
    if email == "fakecrypt@mail.com" {
        Some(CurrentUser {
            email: email.to_string(),
            first_name: "Thanatos".to_string(),
            last_name: "Niet".to_string(),
            password_hash: "".to_string(),
        })
    } else {
        None
    }
}
