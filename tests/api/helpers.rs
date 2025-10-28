use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::OnceLock;
use tokio::net::TcpListener;
use uuid::Uuid;
use zero2prod::configurations::{DatabaseSettings, get_configuration};
use zero2prod::email_client::EmailClient;
use zero2prod::telemetry::{get_subscriber, init_subcriber};

static TRACING: OnceLock<()> = OnceLock::new();

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    TRACING.get_or_init(|| {
        let default_filter_level = "info".to_string();
        let subscriber_name = "test".to_string();

        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
            init_subcriber(subscriber);
        } else {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
            init_subcriber(subscriber);
        }
    });

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let sender = configuration
        .email_client
        .sender()
        .expect("Invalid email address provided");

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender,
        configuration.email_client.authorization_token,
    );

    let connection_pool = configure_databse(&configuration.database).await;

    tokio::spawn(zero2prod::startup::run(
        listener,
        connection_pool.clone(),
        email_client,
    ));

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_databse(config: &DatabaseSettings) -> PgPool {
    // create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");

    connection_pool
}
