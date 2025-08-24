use axum::{Router, http::StatusCode, routing::get};

async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub fn create_app() -> Router {
    Router::new().route("/health", get(health_check))
}

pub async fn run() -> Result<(), std::io::Error> {
    let app = create_app();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await?;

    println!("Server running on http://127.0.0.1:8000");

    axum::serve(listener, app)
        .await
        .map_err(std::io::Error::other)
}
