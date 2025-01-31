use std::sync::Arc;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Trade {
    #[serde(skip_deserializing)]
    id: i32,
    #[sqlx(rename = "title")]
    name: String,
    #[sqlx(rename = "amount")]
    value: f64,
    #[serde(skip_deserializing)]
    date_time: chrono::DateTime<Utc>,
}

impl Trade {
    pub fn sync_date_time(&mut self) {
        self.date_time = Utc::now();
    }
}

pub async fn save(Json(mut trade): Json<Trade>, state: Arc<PgPool>) -> Response {
    trade.sync_date_time();

    let rows_affected =
        sqlx::query(r#"INSERT INTO trades (title, amount, date_time) VALUES ($1, $2, $3)"#)
            .bind(&trade.name)
            .bind(&trade.value)
            .bind(&trade.date_time)
            .execute(state.as_ref())
            .await
            .unwrap()
            .rows_affected();

    if rows_affected < 1 {
        return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    }

    (StatusCode::OK, Json(trade)).into_response()
}

pub async fn get_all(state: Arc<PgPool>) -> Response {
    let trades: Vec<Trade> =
        sqlx::query_as(r#"SELECT id, title, amount, date_time FROM trades WHERE is_active = TRUE"#)
            .fetch_all(state.as_ref())
            .await
            .unwrap();

    for trade in &trades {
        println!("{:?}", trade);
    }

    (StatusCode::OK, Json(trades)).into_response()
}

pub async fn get_by_id(Path(trade_id): Path<i32>, state: Arc<PgPool>) -> Response {
    let trade_finded: Option<Trade> = sqlx::query_as(
        r#"SELECT id, title, amount, date_time FROM trades WHERE id = $1 AND is_active = TRUE"#,
    )
    .bind(trade_id)
    .fetch_optional(state.as_ref())
    .await
    .unwrap();

    match trade_finded {
        Some(trade) => (StatusCode::OK, Json(trade)).into_response(),
        None => (StatusCode::NOT_FOUND).into_response(),
    }
}

pub async fn delete(Path(trade_id): Path<i32>, state: Arc<PgPool>) -> Response {
    let rows_affect = sqlx::query(r#"UPDATE trades SET is_active = FALSE WHERE id = $1"#)
        .bind(trade_id)
        .execute(state.as_ref())
        .await
        .unwrap()
        .rows_affected();

    if rows_affect < 1 {
        return (StatusCode::NOT_MODIFIED).into_response();
    }

    (StatusCode::NO_CONTENT).into_response()
}

pub async fn update(
    Path(trade_id): Path<i32>,
    Json(trade): Json<Trade>,
    state: Arc<PgPool>,
) -> Response {
    let trade_finded: Option<Trade> =
        sqlx::query_as(r#"UPDATE trades SET title = $1, amount = $2 WHERE id = $3 AND is_active = TRUE RETURNING id, title, amount, date_time"#)
            .bind(&trade.name)
            .bind(&trade.value)
            .bind(trade_id)
            .fetch_optional(state.as_ref())
            .await
            .unwrap();

    match trade_finded {
        Some(trade) => (StatusCode::OK, Json(trade)).into_response(),
        None => (StatusCode::NOT_MODIFIED).into_response(),
    }
}
