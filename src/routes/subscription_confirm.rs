use actix_web::web::Query;
use actix_web::{web, HttpResponse};
use sqlx::types::Uuid;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}
#[tracing::instrument(name = "Confirm a pending subscription", skip(params))]
pub async fn confirm(params: Query<Parameters>, pool: web::Data<PgPool>) -> HttpResponse {
    let id = match get_subscriber_from_token(&pool, &params.subscription_token).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    match id {
        None => HttpResponse::Unauthorized().finish(),
        Some(subscriber_id) => {
            if confirm_subscriber(&pool, subscriber_id).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
    }
}

#[tracing::instrument(name = "Get subscriber id from token", skip(pool, token))]
pub async fn get_subscriber_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {}", e);
        e
    })?;

    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(name = "Confirm subscriber", skip(pool, subscriber_id))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' where id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {}", e);
        e
    })?;

    Ok(())
}
