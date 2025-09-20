use sqlx::PgPool;
use tokio::net::TcpListener;
use zero2prod::configurations::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to get configuration.");

    let listener = TcpListener::bind(&address)
        .await
        .expect("Failed to bind to 127.0.0.1:8000");

    run(listener, db_pool).await
}
