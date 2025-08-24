//! tests/health_check.rs

async fn spawn_app() {
    tokio::spawn(async {
        zero2prod::run().await.expect("Failed to start server");
    });
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    spawn_app().await;

    let client = reqwest::Client::new();

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    // Act
    let response = client
        .get("http://127.0.0.1:8000/health")
        .send()
        .await
        .expect("Failed to execute request");

    // Assert

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
