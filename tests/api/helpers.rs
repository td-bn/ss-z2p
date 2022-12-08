use once_cell::sync::Lazy;
use sqlx::{Executor, PgPool};
use uuid::Uuid;
use z2p::configuration::{get_configuration, DatabaseSettings};
use z2p::startup::{get_connection_pool, Application};
use z2p::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let tracing_subscriber = get_subscriber("test".into(), "info".into());
        init_subscriber(tracing_subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscription(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute subscription request")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    // Get config for app start
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(&configuration.clone())
        .await
        .expect("Failed to build application");
    let address = format!("http://127.0.0.1:{}", application.port());
    // Spawn a new task inside tokio runtime
    // tokio's runtime is spun up by actix_rt
    //
    // Cleanup not required as all tokio tasks are dropped when tokio runtime is shut down
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let connection_pool = PgPool::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to postgres");
    println!("DATABASE NAME: {}", config.database_name);
    connection_pool
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate db
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
