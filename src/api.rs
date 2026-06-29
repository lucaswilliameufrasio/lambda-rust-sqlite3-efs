use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures::TryStreamExt;
use serde_json::json;

use crate::{db, db::set_default_env_var, id::generate_xid_string, models::*, sqs};

async fn root() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(
            json!({"message": "API created as an example of how to use EFS with AWS Lambda and store a SQLite database on it"}),
        ),
    )
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"message": "ok"})))
}

async fn load_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MultipleUsersResult>, ApiError> {
    let users_result = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch(&state.pool)
        .map_ok(|u| u)
        .try_collect::<Vec<User>>()
        .await;

    match users_result {
        Ok(users) => Ok(Json(MultipleUsersResult { users })),
        Err(_) => Err(ApiError::SomethingElseWentWrong),
    }
}

async fn find_user(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<User>, ApiError> {
    let users_result = sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = $1")
        .bind(&id)
        .fetch_one(&state.pool)
        .await;

    match users_result {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(ApiError::SomethingElseWentWrong),
    }
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let id = generate_xid_string();
    let queued = QueuedUser::from_create_request(&payload, id.clone());

    let queue_url = match std::env::var("SQS_QUEUE_URL") {
        Ok(url) => url,
        Err(_) => {
            // No queue configured: write directly (local dev fallback)
            let _ = sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $2, $3)")
                .bind(&queued.id)
                .bind(&queued.name)
                .bind(&queued.email)
                .execute(&state.pool)
                .await
                .map_err(|_| ApiError::SomethingWentWrong)?;

            return Ok((
                StatusCode::ACCEPTED,
                Json(json!({ "id": id, "status": "accepted" })),
            ));
        }
    };

    let body = serde_json::to_string(&queued).map_err(|_| ApiError::SomethingWentWrong)?;

    sqs::publish_message(&queue_url, &body).await.map_err(|e| {
        tracing::error!("Failed to publish to SQS: {}", e);
        ApiError::SomethingWentWrong
    })?;

    Ok((
        StatusCode::ACCEPTED,
        Json(json!({ "id": id, "status": "accepted" })),
    ))
}

async fn fallback_handler(uri: Uri) -> impl IntoResponse {
    tracing::error!("No route for {}", uri);
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "message": format!("No route for {}", uri) })),
    )
}

pub fn create_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(root))
        .route("/health-check", get(health_check))
        .route("/users", get(load_users))
        .route("/users/:id", get(find_user))
        .route("/users", post(create_user))
        .fallback(fallback_handler)
}

pub async fn serve_api(state: Arc<AppState>) {
    tracing_subscriber::fmt::init();

    set_default_env_var("PORT", "9989");
    let port = std::env::var("PORT").expect("Application port not defined");

    let address = std::net::SocketAddr::from(([0, 0, 0, 0], port.parse().unwrap()));
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    tracing::info!("API listening on {}", address);

    axum::serve(listener, create_router().with_state(state.clone()))
        .with_graceful_shutdown(db::shutdown_signal(state))
        .await
        .unwrap();
}

enum ApiError {
    SomethingWentWrong,
    SomethingElseWentWrong,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::SomethingWentWrong => {
                (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong")
            }
            ApiError::SomethingElseWentWrong => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "something else went wrong",
            ),
        };
        (status, Json(json!({ "message": body }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use serde_json::Value;
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    #[sqlx::test]
    async fn health_check_should_return_200(pool: SqlitePool) {
        let state = Arc::new(AppState { pool });
        let app = create_router().with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health-check")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({ "message": "ok" }));
    }

    #[sqlx::test]
    async fn root_should_return_200(pool: SqlitePool) {
        let state = Arc::new(AppState { pool });
        let app = create_router().with_state(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            body,
            json!({ "message": "API created as an example of how to use EFS with AWS Lambda and store a SQLite database on it" })
        );
    }

    #[sqlx::test]
    async fn load_users_should_return_200(pool: SqlitePool) {
        let state = Arc::new(AppState { pool });
        let id = generate_xid_string();
        let name = format!("user-{}", &id[..8]);
        let email = format!("{}@example.com", &id[..8]);

        sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $2, $3)")
            .bind(&id)
            .bind(&name)
            .bind(&email)
            .execute(&state.pool)
            .await
            .unwrap();

        let app = create_router().with_state(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();
        let result: MultipleUsersResult = serde_json::from_value(body).unwrap();

        let matched: Vec<&User> = result.users.iter().filter(|u| u.id == id).collect();
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].name, name);
        assert_eq!(matched[0].email, email);
    }

    #[sqlx::test]
    async fn find_user_should_return_200(pool: SqlitePool) {
        let state = Arc::new(AppState { pool });
        let id = generate_xid_string();
        let name = format!("user-{}", &id[..8]);
        let email = format!("{}@example.com", &id[..8]);

        sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $2, $3)")
            .bind(&id)
            .bind(&name)
            .bind(&email)
            .execute(&state.pool)
            .await
            .unwrap();

        let app = create_router().with_state(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/users/{}", id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();
        let user: User = serde_json::from_value(body).unwrap();
        assert_eq!(user.id, id);
        assert_eq!(user.name, name);
        assert_eq!(user.email, email);
    }

    #[sqlx::test]
    async fn create_user_should_return_202(pool: SqlitePool) {
        let state = Arc::new(AppState { pool });
        let app = create_router().with_state(state.clone());

        let user = CreateUserRequest {
            name: "test-user".to_string(),
            email: "test@example.com".to_string(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&user).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert!(body["id"].as_str().unwrap().len() == 24);
        assert_eq!(body["status"], "accepted");
    }

    #[sqlx::test]
    async fn unknown_api_should_be_handled_by_fallback_handler(pool: SqlitePool) {
        let state = Arc::new(AppState { pool });
        let app = create_router().with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/does-not-exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({ "message": "No route for /does-not-exist" }));
    }
}
