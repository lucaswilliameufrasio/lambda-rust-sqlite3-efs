use std::{fs, sync::Arc};
use tracing::log::LevelFilter;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePool},
    ConnectOptions,
};

use crate::models::AppState;

pub const DEFAULT_DATABASE_URL: &str = "sqlite:users.db";
pub const DEFAULT_DATABASE_PATH: &str = "./users.db";

pub async fn bootstrap() -> Arc<AppState> {
    dotenv::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    let database_path =
        std::env::var("DATABASE_PATH").unwrap_or_else(|_| DEFAULT_DATABASE_PATH.to_string());

    let file_metadata = fs::metadata(&database_path);
    if file_metadata.is_err() {
        let _ = fs::File::create(&database_path);
    }

    let connection_options: SqliteConnectOptions = database_url.parse().unwrap();
    let pool = SqlitePool::connect_with(connection_options.log_statements(LevelFilter::Off))
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!().run(&pool).await.unwrap();

    let _ = sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(&pool)
        .await;

    let _ = sqlx::query("PRAGMA busy_timeout = 5000;")
        .execute(&pool)
        .await;

    Arc::new(AppState { pool })
}

pub fn set_default_env_var(key: &str, value: &str) {
    if std::env::var(key).is_err() {
        std::env::set_var(key, value);
    }
}

pub async fn create_pool(database_url: &str) -> SqlitePool {
    let connection_options: SqliteConnectOptions = database_url.parse().unwrap();
    SqlitePool::connect_with(connection_options.log_statements(LevelFilter::Off))
        .await
        .expect("Failed to connect to database")
}

pub async fn run_migrations(pool: &SqlitePool) {
    sqlx::migrate!().run(pool).await.unwrap();
}

pub async fn shutdown_signal(state: Arc<AppState>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
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
