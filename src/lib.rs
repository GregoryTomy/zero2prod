use axum::{Router, http::StatusCode, routing::get};
use tokio::net::TcpListener;

async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub fn create_app() -> Router {
    Router::new().route("/health", get(health_check))
}

pub async fn run(listener: TcpListener) -> Result<(), std::io::Error> {
    let app = create_app();

    println!("Server running on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .map_err(std::io::Error::other)
}
