use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[actix_rt::test]
async fn subscribe_returns_200_for_valid_data() {
    // Arrange
    let test_app = spawn_app().await;

    let body = "name=bn&email=tdnb%40hello.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;
    let response = test_app.post_subscription(body.to_string()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[actix_rt::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let test_app = spawn_app().await;

    // DB
    let connection = test_app.db_pool.clone();
    let body = "name=bn&email=tdnb%40hello.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let response = test_app.post_subscription(body.to_string()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&connection)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "tdnb@hello.com");
    assert_eq!(saved.name, "bn");
    assert_eq!(saved.status, "pending_confirmation");
}

#[actix_rt::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20me", "missing the email"),
        ("email=le%40me.com", "missing the name"),
        ("", "missing the name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscription(invalid_body.to_string()).await;
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
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=&email=tdbn@gmail.com", "empty name"),
        ("name=tdbn&email=", "empty email"),
        ("name=tdbn&email=not-an-email", "invalid email"),
    ];

    for (body, _description) in test_cases {
        let response = app.post_subscription(body.to_string()).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 with payload {}",
            body
        )
    }
}

#[actix_rt::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;

    let body = "name=bn&email=tdnb%40hello.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscription(body.to_string()).await;

    // Assert
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request).await;

    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}

#[actix_rt::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    // Arrange
    let app = spawn_app().await;

    let body = "name=bn&email=tdnb%40hello.com";

    // Sabotage! HAHAH Evil Laugh!
    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;")
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app.post_subscription(body.into()).await;

    // Assert
    assert_eq!(response.status().as_u16(), 500);
}
