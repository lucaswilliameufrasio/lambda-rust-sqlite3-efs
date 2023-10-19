use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, net::SocketAddr, sync::Arc};
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
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:users.db".to_string());
    let database_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./users.db".to_string());

    let file_metadata = fs::metadata(database_path.clone());

    match file_metadata {
        Ok(_) => {}
        Err(_) => {
            let _ = fs::File::create(database_path);
        }
    }

    let mut connection_options: SqliteConnectOptions = database_url.parse().unwrap();
    connection_options.log_statements(LevelFilter::Off);

    let pool = SqlitePool::connect_with(connection_options)
        .await
        .expect("Failed to connect to database");

    let state = Arc::new(AppState { pool });

    state
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // initialize tracing
    tracing_subscriber::fmt::init();

    let state = bootstrap().await;

    sqlx::migrate!().run(&state.pool).await.unwrap();

    let _ = sqlx::query("pragma journal_mode = WAL;")
        .execute(&state.pool)
        .await;

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/users", get(load_users))
        .route("/users", post(create_user))
        .with_state(state);

    set_default_env_var("PORT", "9989");

    let port = std::env::var("PORT").expect("Application port not defined");

    let address = SocketAddr::from(([0, 0, 0, 0], port.parse().unwrap()));

    tracing::debug!("Started listening on {}", address);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "message": "OK" })))
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

#[derive(Serialize)]
struct MultipleUsersResult {
    users: Vec<User>,
}

async fn load_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MultipleUsersResult>, MyError> {
    let users_result = sqlx::query_as!(
        UserFromQuery,
        r#"select id as "id: i64", name, email from users"#
    )
    .fetch(&state.pool)
    .map_ok(UserFromQuery::into_user)
    .try_collect()
    .await;

    match users_result {
        Ok(users) => Ok(Json(MultipleUsersResult { users: users })),
        Err(_) => Err(MyError::SomethingElseWentWrong),
    }
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), MyError> {
    match sqlx::query(
        r#"
            INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *;
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .fetch_one(&state.pool)
    .await
    {
        Ok(user) => {
            return Ok((
                StatusCode::CREATED,
                Json(User {
                    id: user.get("id"),
                    name: payload.name,
                    email: payload.email,
                }),
            ));
        }
        Err(error) => {
            println!("{}", error);
            return Err(MyError::SomethingWentWrong);
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Sqlite>,
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

// the output to our `create_user` handler
#[derive(Serialize, Debug)]
struct User {
    id: i64,
    name: String,
    email: String,
}

enum MyError {
    SomethingWentWrong,
    SomethingElseWentWrong,
}

impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        let body = match self {
            MyError::SomethingWentWrong => "something went wrong",
            MyError::SomethingElseWentWrong => "something else went wrong",
        };

        // its often easiest to implement `IntoResponse` by calling other implementations
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
