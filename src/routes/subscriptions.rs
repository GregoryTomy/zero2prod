use axum::{Form, extract::State, http::StatusCode};
use serde::Deserialize;
use sqlx::{PgPool, types::chrono::Utc};
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;

        Ok(Self { email, name })
    }
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
        new_subscriber.email.as_ref(),
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
    let new_subscriber = match form.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match insert_subsriber(&pool, &new_subscriber).await {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved");
            StatusCode::OK
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
