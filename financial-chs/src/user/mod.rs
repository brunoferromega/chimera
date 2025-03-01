use std::{mem, sync::Arc};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[allow(dead_code)]
#[derive(Deserialize, Clone)]
pub struct User {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    password: String,
}

impl User {
    fn take_password(&mut self) -> String {
        mem::take(&mut self.password)
    }
}

#[derive(Clone, Serialize, sqlx::FromRow)]
pub struct UserOut {
    pub email: String,
    pub username: String,
    #[serde(skip_deserializing, skip_serializing)]
    pub hash_password: String,
}

impl From<User> for UserOut {
    fn from(user: User) -> Self {
        UserOut {
            email: user.email,
            username: format!(
                "{first}+{last}",
                first = &user.first_name,
                last = &user.last_name
            ),
            hash_password: String::default(),
        }
    }
}

impl UserOut {
    fn add_hash_password(self, password: String) -> Option<Self> {
        match hash(password, DEFAULT_COST) {
            Ok(hash_password) => Some(UserOut {
                email: self.email,
                username: self.username,
                hash_password,
            }),
            Err(_) => None,
        }
    }
}

pub async fn register(Json(mut user): Json<User>, state: Arc<PgPool>) -> Response {
    if user.password.is_empty() {
        return (StatusCode::BAD_REQUEST).into_response();
    }

    let password = user.take_password();
    let user_out = UserOut::from(user);
    let user_out = match user_out.add_hash_password(password.to_string()) {
        Some(user_o) => user_o,
        None => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

    let rows_affected =
        sqlx::query(r#"INSERT INTO users (email, username, hash_password) VALUES ($1, $2, $3) "#)
            .bind(&user_out.email)
            .bind(&user_out.username)
            .bind(&user_out.hash_password)
            .execute(state.as_ref())
            .await
            .unwrap()
            .rows_affected();

    if rows_affected < 1 {
        return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    }

    (StatusCode::OK, Json(user_out)).into_response()
}

pub async fn find(email: String, db: Arc<PgPool>) -> Option<UserOut> {
    let user: Result<UserOut, _> =
        sqlx::query_as(r#"SELECT email, username, hash_password FROM users WHERE email = $1"#)
            .bind(email)
            .fetch_one(db.as_ref())
            .await;

    match user {
        Ok(user) => Some(user),
        Err(_) => None,
    }
}
