use crate::helpers::spawn_app;

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

#[actix_rt::test]
async fn subscribe_returns_400_when_data_is_empty() {
    let address = spawn_app().await.address;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=&email=tdbn@gmail.com", "empty name"),
        ("name=tdbn&email=", "empty email"),
        ("name=tdbn&email=not-an-email", "invalid email"),
    ];

    for (body, _description) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute subscription request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 with payload {}",
            body
        )
    }
}
