use std::net::TcpListener;

#[actix_rt::test]
async fn health_check_works() {
    let address = spawn_app();

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
    let address = spawn_app();

    let client = reqwest::Client::new();
    let body = "name=bn&email=tdnb%40hello.com";

    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute subscription request");

    assert_eq!(200, response.status().as_u16());
}

#[actix_rt::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let address = spawn_app();

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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    // Retreive the port
    let port = listener.local_addr().unwrap().port();

    let server = z2p::run(listener).expect("Failed to start server");
    // Spawn a new task inside tokio runtime
    // tokio's runtime is spun up by actix_rt
    //
    // Cleanup not required as all tokio tasks are dropped when tokio runtime is shut down
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
