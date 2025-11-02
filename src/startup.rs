use crate::configurations::Settings;
use crate::email_client::EmailClient;
use crate::routes::health_check;
use crate::routes::subscribe;
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub struct Application {
    port: u16,
    listener: TcpListener,
    router: Router,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = PgPool::connect_lazy_with(configuration.database.with_db());

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");

        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(&address)
            .await
            .expect(&format!("Failed to bind to {}", address));

        let port = listener.local_addr()?.port();

        let router = create_app(connection_pool, email_client);

        Ok(Self {
            port,
            listener,
            router,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        tracing::info!("Server listening on port {}", self.port);

        axum::serve(self.listener, self.router).await
    }
}

pub fn create_app(db_pool: PgPool, email_client: EmailClient) -> Router {
    let email_client = Arc::new(email_client);

    Router::new()
        .route("/health", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(TraceLayer::new_for_http())
        .with_state((db_pool, email_client))
}
