use tokio::net::TcpListener;
use zero2prod::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:8000")
        .await
        .expect("Failed to bind to 127.0.0.1:8000");
    run(listener).await
}
