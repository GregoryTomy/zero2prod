use crate::routes::health_check;
use crate::routes::subscribe;
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use tokio::net::TcpListener;

pub fn create_app(db_pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(db_pool)
}

pub async fn run(listener: TcpListener, db_pool: PgPool) -> Result<(), std::io::Error> {
    let app = create_app(db_pool);

    println!("Server running on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .map_err(std::io::Error::other)
}
