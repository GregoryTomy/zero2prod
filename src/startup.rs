use crate::email_client::EmailClient;
use crate::routes::health_check;
use crate::routes::subscribe;
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub fn create_app(db_pool: PgPool, email_client: EmailClient) -> Router {
    let email_client = Arc::new(email_client);

    Router::new()
        .route("/health", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(TraceLayer::new_for_http())
        .with_state((db_pool, email_client))
}

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<(), std::io::Error> {
    let app = create_app(db_pool, email_client);

    tracing::info!("Server running on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .map_err(std::io::Error::other)
}
