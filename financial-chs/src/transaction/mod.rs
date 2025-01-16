use std::str::FromStr;

use serde::{Serialize, Deserialize};
use chrono::prelude::*;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    name: String,
    value: f64,
    #[serde(skip_deserializing)]
    date_time: chrono::DateTime<Utc>,
}

impl Trade {
    pub fn sync_date_time(&mut self) {
        self.date_time = Utc::now();
    }
}

pub async fn save_t(Json(mut trade): Json<Trade>) -> Response {
    trade.sync_date_time();
    (StatusCode::OK, Json(trade)).into_response() 
}
