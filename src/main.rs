use sqlx::PgPool;
use tokio::net::TcpListener;
use zero2prod::configurations::get_configuration;
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subcriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);

    init_subcriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let db_pool = PgPool::connect_lazy_with(configuration.database.with_db());

    let listener = TcpListener::bind(&address)
        .await
        .expect("Failed to bind to 127.0.0.1:8000");

    run(listener, db_pool).await
}
