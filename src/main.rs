use lambda_rust_sqlite3_efs::{api, db};

#[tokio::main]
async fn main() {
    let state = db::bootstrap().await;
    api::serve_api(state).await;
}
