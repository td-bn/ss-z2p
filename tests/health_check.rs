use sqlx::{Executor, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use z2p::configuration::{get_configuration, DatabaseSettings};
use z2p::startup::run;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[actix_rt::test]
async fn health_check_works() {
    let test_app = spawn_app().await;

    let address = test_app.address;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn subscribe_returns_200_for_valid_data() {
    // Arrange
    let test_app = spawn_app().await;
    let address = test_app.address;

    // DB
    let connection = test_app.db_pool.clone();
    let client = reqwest::Client::new();
    let body = "name=bn&email=tdnb%40hello.com";

    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute subscription request");

    // Assert
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&connection)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "tdnb@hello.com");
    assert_eq!(saved.name, "bn");
}

#[actix_rt::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let address = spawn_app().await.address;

    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20me", "missing the email"),
        ("email=le%40me.com", "missing the name"),
        ("", "missing the name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute subscription request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 bad Request with payload {}",
            error_message
        );
    }
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    // Retrieve the port
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // Get config for app start
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to start server");
    // Spawn a new task inside tokio runtime
    // tokio's runtime is spun up by actix_rt
    //
    // Cleanup not required as all tokio tasks are dropped when tokio runtime is shut down
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
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
