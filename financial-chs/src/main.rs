use std::env;
use std::error::Error;
use std::sync::Arc;

use axum::{
    routing::get,
    Router,
};

mod db;
mod trade;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conn_str = env::var("DB_URL").expect("Please input database url when run the program");

    let pool = sqlx::PgPool::connect(&conn_str).await?;

    let shared_pool = Arc::new(pool);

    let health_ep = Router::new().route("/", get(|| async { "I am alive" }));

    let trade_ep = Router::new()
        .route(
            "/",
            get({
                let shared_pool = Arc::clone(&shared_pool);
                move || trade::get_all(shared_pool)
            })
            .post({
                let shared_pool = Arc::clone(&shared_pool);
                move |body| trade::save(body, shared_pool)
            }),
        )
        .route(
            "/{id}",
            get({
                let shared_pool = Arc::clone(&shared_pool);
                move |path| trade::get_by_id(path, shared_pool)
            }),
        );

    let api_eps = Router::new()
        .nest("/health", health_ep)
        .nest("/trade", trade_ep);

    let app = Router::new().nest("/api", api_eps);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
