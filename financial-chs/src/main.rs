use std::env;
use std::error::Error;
use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

mod auth;
mod db;
mod trade;
mod user;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conn_str = env::var("DB_URL").expect("Please input database url when run the program");

    let pool = sqlx::PgPool::connect(&conn_str).await?;

    let shared_pool = Arc::new(pool);

    let health_rt = Router::new().route("/", get(|| async { "I am alive" }));

    let auth_rt = Router::new().route(
        "/",
        post({
            let shared_pool = Arc::clone(&shared_pool);
            move |body| auth::sign_in(body, shared_pool)
        }),
    );

    let user_rt = Router::new().route(
        "/",
        post({
            let shared_pool = Arc::clone(&shared_pool);
            move |body| user::register(body, shared_pool)
        }),
    );

    let trade_rt = Router::new()
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
            })
            .delete({
                let shared_pool = Arc::clone(&shared_pool);
                move |path| trade::delete(path, shared_pool)
            })
            .put({
                let shared_pool = Arc::clone(&shared_pool);
                move |path: axum::extract::Path<_>, body: axum::Json<_>| {
                    trade::update(path, body, shared_pool)
                }
            }),
        )
        .route_layer(middleware::from_fn_with_state(
            Arc::clone(&shared_pool),
            auth::authorize,
        ));

    let api_eps = Router::new()
        .nest("/health", health_rt)
        .nest("/trade", trade_rt)
        .nest("/sign_in", auth_rt)
        .nest("/login", user_rt);

    let app = Router::new().nest("/api", api_eps);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
