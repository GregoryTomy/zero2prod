use zero2prod::configurations::get_configuration;
use zero2prod::startup::Application;

use zero2prod::telemetry::{get_subscriber, init_subcriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);

    init_subcriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    let app = Application::build(configuration).await?;

    app.run_until_stopped().await
}
