use axum::{Router, extract::Path, routing::get};

async fn greet() -> &'static str {
    "Hello World!"
}

async fn greet_name(Path(name): Path<String>) -> String {
    format!("Hello {name}!")
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(greet))
        .route("/{name}", get(greet_name));

    let listner = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:8000");

    axum::serve(listner, app).await.unwrap();
}
