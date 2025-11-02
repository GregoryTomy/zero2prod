use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::OnceLock;
use uuid::Uuid;
use zero2prod::configurations::{DatabaseSettings, get_configuration};
use zero2prod::startup::Application;
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

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    configuration.application.port = 0;

    let connection_pool = configure_databse(&configuration.database).await;

    let app = Application::build(configuration)
        .await
        .expect("Failed to build application");

    let address = format!("http://127.0.0.1:{}", app.port());

    tokio::spawn(app.run_until_stopped());

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
