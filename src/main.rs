use axum::{
    extract::State,
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, net::SocketAddr, sync::Arc};
use tokio::signal;
use tracing::log::LevelFilter;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePool},
    ConnectOptions, Pool, Row, Sqlite,
};

fn set_default_env_var(key: &str, value: &str) {
    if std::env::var(key).is_err() {
        std::env::set_var(key, value);
    }
}

async fn bootstrap() -> Arc<AppState> {
    dotenv::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:users.db".to_string());
    let database_path: String =
        std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./users.db".to_string());

    let file_metadata = fs::metadata(database_path.clone());

    match file_metadata {
        Ok(_) => {}
        Err(_) => {
            let _ = fs::File::create(database_path);
        }
    }

    let connection_options: SqliteConnectOptions = database_url.parse().unwrap();

    let pool = SqlitePool::connect_with(connection_options.log_statements(LevelFilter::Off))
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!().run(&pool).await.unwrap();

    Arc::new(AppState { pool })
}

async fn shutdown_signal(state: Arc<AppState>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("Closing all remaining connections after CTRL+C");
            state.pool.close().await;
        },
        _ = terminate => {
            println!("Closing all remaining connections after SIGTERM");
            state.pool.close().await;
        },
    }

    println!("signal received, starting graceful shutdown");
}

fn app() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(root))
        .route("/health-check", get(health_check))
        .route("/users", get(load_users))
        .route("/users", post(create_user))
        .fallback(fallback_handler)
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let state = bootstrap().await;

    let _ = sqlx::query("pragma journal_mode = WAL;")
        .execute(&state.pool)
        .await;

    set_default_env_var("PORT", "9989");

    let port = std::env::var("PORT").expect("Application port not defined");

    let address = SocketAddr::from(([0, 0, 0, 0], port.parse().unwrap()));
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    tracing::info!("Listening on {}", address);
    axum::serve(listener, app().with_state(state.clone()))
        .with_graceful_shutdown(shutdown_signal(state))
        .await
        .unwrap();
}

async fn fallback_handler(uri: Uri) -> impl IntoResponse {
    tracing::error!("No route for {}", uri);
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "message": format!("No route for {}", uri) })),
    )
}

async fn root() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(
            json!({ "message": "API created as an example of how to use EFS with AWS Lambda and store a SQLite database on it" }),
        ),
    )
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "message": "ok" })))
}

#[derive(sqlx::FromRow)]
struct UserFromQuery {
    id: i64,
    name: String,
    email: String,
}

impl UserFromQuery {
    fn into_user(self) -> User {
        User {
            id: self.id,
            name: self.name,
            email: self.email,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct MultipleUsersResult {
    users: Vec<User>,
}

async fn load_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MultipleUsersResult>, APIError> {
    let users_result = sqlx::query_as!(
        UserFromQuery,
        r#"select id as "id: i64", name, email from users"#
    )
    .fetch(&state.pool)
    .map_ok(UserFromQuery::into_user)
    .try_collect()
    .await;

    match users_result {
        Ok(users) => Ok(Json(MultipleUsersResult { users })),
        Err(_) => Err(APIError::SomethingElseWentWrong),
    }
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), APIError> {
    match sqlx::query(
        r#"
            INSERT INTO users (name, email) VALUES ($1, $2) RETURNING users.id;
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .fetch_one(&state.pool)
    .await
    {
        Ok(user) => Ok((
            StatusCode::CREATED,
            Json(User {
                id: user.get("id"),
                name: payload.name,
                email: payload.email,
            }),
        )),
        Err(error) => {
            tracing::error!("Failed to create user {}", error);
            Err(APIError::SomethingWentWrong)
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Sqlite>,
}

// the input to our `create_user` handler
#[derive(Serialize, Deserialize, Clone)]
struct CreateUser {
    name: String,
    email: String,
}

// the output to our `create_user` handler
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct User {
    id: i64,
    name: String,
    email: String,
}

enum APIError {
    SomethingWentWrong,
    SomethingElseWentWrong,
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        let body = match self {
            APIError::SomethingWentWrong => "something went wrong",
            APIError::SomethingElseWentWrong => "something else went wrong",
        };

        // its often easiest to implement `IntoResponse` by calling other implementations
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    // for `collect`
    use serde_json::{json, Value};
    use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

    #[tokio::test]
    async fn health_check_should_return_200() {
        let state = bootstrap().await;

        let app = app().with_state(state.clone());

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

    #[tokio::test]
    async fn root_should_return_200() {
        let state = bootstrap().await;

        let app = app().with_state(state.clone());

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

        let user_to_be_created = CreateUser {
            name: nanoid::nanoid!().to_string(),
            email: format!("{}@example.com", nanoid::nanoid!().to_string()),
        };

        let created_user_row = sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING users.id, users.name, users.email;")
            .bind(&user_to_be_created.name)
            .bind(&user_to_be_created.email)
            .fetch_one(&state.pool) // Execute the query using the acquired connection
            .await
            .unwrap();

        let app = app().with_state(state.clone());

        let created_user = User {
            id: created_user_row.get("id"),
            name: created_user_row.get("name"),
            email: created_user_row.get("email"),
        };

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
        let multiple_users_result: MultipleUsersResult = serde_json::from_value(body).unwrap();

        let expected_users: Vec<User> = multiple_users_result
            .users
            .iter()
            .cloned()
            .filter(|user| user.id.eq(&created_user.id))
            .collect();

        assert_eq!(expected_users.clone().len(), 1);
        assert_eq!(*expected_users.first().unwrap(), created_user);
    }

    #[sqlx::test]
    async fn create_user_should_return_201(pool: SqlitePool) {
        let state = Arc::new(AppState { pool });

        let app = app().with_state(state.clone());

        let user = CreateUser {
            name: nanoid::nanoid!(),
            email: format!("{}@example.com", nanoid::nanoid!()),
        };

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
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

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();
        let the_created_user: User = serde_json::from_value(body).unwrap();
        assert_eq!(the_created_user.name, user.name);
        assert_eq!(the_created_user.email, user.email);
    }

    #[tokio::test]
    async fn unknown_api_should_be_handled_by_fallback_handler() {
        let state = bootstrap().await;

        let app = app().with_state(state.clone());

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
