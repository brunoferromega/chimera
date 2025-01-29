use std::str::FromStr;
use std::sync::Arc;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Trade {
    #[serde(skip_deserializing)]
    id: u64,
    #[sqlx(rename = "title")]
    name: String,
    #[slqx(rename = "amount")]
    value: f64,
    #[serde(skip_deserializing)]
    date_time: chrono::DateTime<Utc>,
}

impl Trade {
    pub fn sync_date_time(&mut self) {
        self.date_time = Utc::now();
    }
}

pub async fn save_trade(Json(mut trade): Json<Trade>, state: Arc<PgPool>) -> Response {
    dbg!(&trade);
    dbg!(&state);
    trade.sync_date_time();

    let _rows_affected =
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

pub async fn get_all_trades(state: Arc<PgPool>) -> Response {
    let trades: Vec<Trade> = sqlx::query_as(r#"SELECT * FROM trades"#)
        .fetch_all(state.as_ref())
        .await
        .unwrap();

    (StatusCode::OK, Json(trades)).into_response()
}
