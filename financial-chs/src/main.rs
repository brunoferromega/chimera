use std::error::Error;

use axum::{
    routing::{get, post, delete},
    Router,
};

mod db;
mod transaction;

#[tokio::main]
async fn main() -> Result<(),Box<dyn Error>> {
    let app = Router::new()
        .route("/api/health", get(|| async { "I am alive" }))
        .route("/api/trade", post(transaction::save_t));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
