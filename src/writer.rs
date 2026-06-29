use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde_json::json;
use sqlx::Pool;

use crate::{db, models::*};

async fn handle_events(
    State(state): State<Arc<AppState>>,
    body: String,
) -> Result<impl IntoResponse, WriterError> {
    let event: SqsEvent = serde_json::from_str(&body).map_err(|e| {
        tracing::error!("Failed to parse SQS event: {}", e);
        WriterError::BadRequest
    })?;

    let mut processed = 0u32;
    let mut failed = 0u32;

    for record in &event.records {
        let message_body = match &record.body {
            Some(b) => b,
            None => {
                tracing::warn!("SQS record has no body, skipping");
                continue;
            }
        };

        let queued: QueuedUser = match serde_json::from_str(message_body) {
            Ok(u) => u,
            Err(e) => {
                tracing::error!("Failed to parse queued user: {}", e);
                failed += 1;
                continue;
            }
        };

        match insert_user(&state.pool, &queued).await {
            Ok(_) => {
                processed += 1;
                tracing::info!("Inserted user {} ({})", queued.id, queued.name);
            }
            Err(e) => {
                tracing::error!("Failed to insert user {}: {}", queued.id, e);
                failed += 1;
            }
        }
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "processed": processed,
            "failed": failed,
        })),
    ))
}

async fn insert_user(pool: &Pool<sqlx::Sqlite>, user: &QueuedUser) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $2, $3)")
        .bind(&user.id)
        .bind(&user.name)
        .bind(&user.email)
        .execute(pool)
        .await?;

    Ok(())
}

pub fn create_router() -> Router<Arc<AppState>> {
    Router::new().route("/events", post(handle_events))
}

pub async fn serve_writer(state: Arc<AppState>) {
    tracing_subscriber::fmt::init();

    db::set_default_env_var("PORT", "9988");
    let port = std::env::var("PORT").expect("Application port not defined");

    let address = std::net::SocketAddr::from(([0, 0, 0, 0], port.parse().unwrap()));
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    tracing::info!("Writer listening on {}", address);

    axum::serve(listener, create_router().with_state(state.clone()))
        .with_graceful_shutdown(db::shutdown_signal(state))
        .await
        .unwrap();
}

enum WriterError {
    BadRequest,
}

impl IntoResponse for WriterError {
    fn into_response(self) -> axum::response::Response {
        let (status, body) = match self {
            WriterError::BadRequest => (StatusCode::BAD_REQUEST, "invalid event payload"),
        };
        (status, Json(json!({ "message": body }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parse_valid_sqs_event() {
        let body = r#"{
            "Records": [{
                "messageId": "1",
                "receiptHandle": "abc",
                "body": "{\"id\":\"abc123\",\"name\":\"test\",\"email\":\"test@example.com\"}",
                "attributes": {},
                "messageAttributes": {},
                "md5OfBody": "xyz",
                "eventSource": "aws:sqs",
                "eventSourceARN": "arn:aws:sqs:us-east-1:000000000000:queue",
                "awsRegion": "us-east-1"
            }]
        }"#;

        let event: SqsEvent = serde_json::from_str(body).unwrap();
        assert_eq!(event.records.len(), 1);
        let record = &event.records[0];
        assert_eq!(record.event_source.as_deref(), Some("aws:sqs"));

        let queued: QueuedUser = serde_json::from_str(record.body.as_ref().unwrap()).unwrap();
        assert_eq!(queued.id, "abc123");
        assert_eq!(queued.name, "test");
    }

    #[sqlx::test]
    async fn insert_user_into_db(pool: sqlx::SqlitePool) {
        let user = QueuedUser {
            id: "test-id-001".to_string(),
            name: "integration".to_string(),
            email: "int@example.com".to_string(),
        };

        insert_user(&pool, &user).await.unwrap();

        let row = sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = $1")
            .bind(&user.id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(row.id, user.id);
        assert_eq!(row.name, user.name);
        assert_eq!(row.email, user.email);
    }
}
