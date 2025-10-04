use axum::{Form, extract::State, http::StatusCode};
use serde::Deserialize;
use sqlx::{PgPool, types::chrono::Utc};
use tracing::Instrument;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(State(pool): State<PgPool>, Form(form): Form<FormData>) -> StatusCode {
    let request_id = Uuid::new_v4();

    let request_span = tracing::info_span!(
        "Adding new subscriber",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    );

    let _request_span_gaurd = request_span.enter();

    tracing::info!(
        "request_id {} - Adding '{}' '{}' as a new subscriber",
        request_id,
        form.email,
        form.name
    );

    tracing::info!(
        "request_id {} - Saving new subscriber details to database",
        request_id,
    );

    let query_span = tracing::info_span!("Saving new subscriber details in database");

    match sqlx::query!(
        r#"
        insert into subscriptions (id, email, name, subscribed_at)
        values ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(&pool)
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New subscriber details have been saved",
                request_id,
            );
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("request_id {} - Failed to execute query: {e:?}", request_id);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
