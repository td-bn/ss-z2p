use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use z2p::configuration::get_configuration;
use z2p::startup::run;
use z2p::telemetry::*;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let tracing_subscriber = get_subscriber("Z2P".into(), "info".into());
    init_subscriber(tracing_subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    let connection = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect(&configuration.database.connection_string())
        .await
        .unwrap_or_else(|_| {
            panic!(
                "Failed to connect to Postgres with connection string: {}",
                &configuration.database.connection_string().as_str()
            )
        });
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}
