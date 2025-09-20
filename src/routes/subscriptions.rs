use axum::{Form, extract::State, http::StatusCode};
use serde::Deserialize;
use sqlx::{PgPool, types::chrono::Utc};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(State(pool): State<PgPool>, Form(form): Form<FormData>) -> StatusCode {
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
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            println!("Failed to execute query{e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
