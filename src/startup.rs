use crate::routes::health_check;
use crate::routes::subscribe;
use axum::{
    Router,
    routing::{get, post},
};
use tokio::net::TcpListener;

pub fn create_app() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/subscriptions", post(subscribe))
}

pub async fn run(listener: TcpListener) -> Result<(), std::io::Error> {
    let app = create_app();

    println!("Server running on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .map_err(std::io::Error::other)
}
