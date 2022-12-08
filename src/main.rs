use z2p::configuration::get_configuration;
use z2p::startup::Application;
use z2p::telemetry::*;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let tracing_subscriber = get_subscriber("Z2P".into(), "info".into());
    init_subscriber(tracing_subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let application = Application::build(&configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
