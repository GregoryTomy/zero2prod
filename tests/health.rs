//! tests/health_check.rs

use sqlx::PgPool;
use tokio::net::TcpListener;
use zero2prod::configurations::get_configuration;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    tokio::spawn(zero2prod::startup::run(listener, connection_pool.clone()));

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    let address = app.address;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health", &address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=rae%20boone&email=rae_boone%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("select email, name from subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "rae_boone@gmail.com");
    assert_eq!(saved.name, "rae boone")
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=rae%20boone", "missing email"),
        ("email=rae_boone%40gmail.com", "missing name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when payload was {}",
            error_message
        )
    }
}
