use axum::{Form, extract::State, http::StatusCode};
use serde::Deserialize;
use sqlx::{PgPool, types::chrono::Utc};
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberName};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Saving new subscriber details in database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subsriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        insert into subscriptions (id, email, name, subscribed_at)
        values ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email,
        new_subscriber.name.as_ref(),
        Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(
    name="Adding new subscriber",
    skip(form, pool),
    fields(
        request_id=%Uuid::new_v4(),
        subscriber_email=%form.email,
        subscriber_name=%form.name,
    )
)]
pub async fn subscribe(State(pool): State<PgPool>, Form(form): Form<FormData>) -> StatusCode {
    let name = match SubscriberName::parse(form.name) {
        Ok(name) => name,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let new_subscriber = NewSubscriber {
        email: form.email,
        name,
    };

    match insert_subsriber(&pool, &new_subscriber).await {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved");
            StatusCode::OK
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
