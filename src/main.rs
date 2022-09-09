use sqlx::PgPool;
use std::net::TcpListener;
use z2p::configuration::get_configuration;
use z2p::startup::run;
use z2p::telemetry::*;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let tracing_subscriber = get_subscriber("Z2P".into(), "info".into());
    init_subscriber(tracing_subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}
