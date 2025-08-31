use axum::{
    Form, Router,
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;
use tokio::net::TcpListener;

#[allow(dead_code)]
#[derive(Deserialize)]
struct FormData {
    email: String,
    name: String,
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn subscribe(_form: Form<FormData>) -> StatusCode {
    StatusCode::OK
}

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
