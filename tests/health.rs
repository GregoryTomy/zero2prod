//! tests/health_check.rs

use tokio::net::TcpListener;

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async {
        zero2prod::run(listener)
            .await
            .expect("Failed to start server");
    });

    format!("http://127.0.0.1:{port}")
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Act
    let response = client
        .get(format!("{}/health", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
