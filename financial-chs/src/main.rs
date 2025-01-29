use std::error::Error;
use std::sync::Arc;
use std::env;

use axum::{
    routing::{delete, get, post},
    Router,
};

use sqlx::PgPool;

mod db;
mod transaction;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conn_str = env::var("DB_URL").expect("Please input database url when run the program");

    let pool = sqlx::PgPool::connect(&conn_str).await?;

    let shared_pool = Arc::new(pool);

    let app = Router::new()
        .route("/api/health", get(|| async { "I am alive" }))
        .route(
            "/api/trade",
            get({
                let shared_pool = Arc::clone(&shared_pool);
                move || transaction::get_all_trades(shared_pool);
            })
            .post({
                let shared_pool = Arc::clone(&shared_pool);
                move |body| transaction::save_trade(body, shared_pool)
            }),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
