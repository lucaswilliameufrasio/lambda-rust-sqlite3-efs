use lambda_rust_sqlite3_efs::{db, writer};

#[tokio::main]
async fn main() {
    let state = db::bootstrap().await;
    writer::serve_writer(state).await;
}
