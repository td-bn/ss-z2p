use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::routes::error_chain_fmt;
use actix_http::header::HeaderMap;
use actix_http::{header, StatusCode};
use actix_web::http::header::HeaderValue;
use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::PgPool;
use std::fmt::{Debug, Formatter};

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    text: String,
    html: String,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Authentication Failed")]
    AuthError(#[source] anyhow::Error),
}

impl Debug for PublishError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
            PublishError::AuthError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                response
                    .headers_mut()
                    .insert(header::WWW_AUTHENTICATE, header_value);
                response
            }
        }
    }
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let _credentials = basic_authentication(request.headers())
        .await
        .map_err(PublishError::AuthError)?;
    let confirmed_subscribers = get_confirmed_subscriber(&pool).await?;
    for subscriber in confirmed_subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_mail(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send a newsletter to {}", subscriber.email)
                    })?;
            }
            Err(e) => {
                tracing::warn!(
                    e.cause_chain = ?e,
                    "Skipping a confirmed subscriber\
                    Their stored email is invalid"
                );
            }
        }
    }
    Ok(HttpResponse::Ok().finish())
}

struct Credentials {
    _username: String,
    _password: String,
}

async fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF-8 string")?;

    let b64_segment = header
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'")?;

    let decoded_bytes = base64::decode_config(b64_segment, base64::STANDARD)
        .context("Failed to B64 decode 'Basic' credentials")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("Decoded B64 credential string is not valid UTF-8")?;

    let mut credentials = decoded_credentials.splitn(2, ':');

    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be present in 'Basic' auth"))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be present in 'Basic' auth"))?
        .to_string();

    Ok(Credentials {
        _username: username,
        _password: password,
    })
}

#[tracing::instrument(name = "Getting confirmed subscriber list", skip(pool))]
async fn get_confirmed_subscriber(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(e) => Err(anyhow::anyhow!(e)),
    })
    .collect();
    Ok(confirmed_subscribers)
}
