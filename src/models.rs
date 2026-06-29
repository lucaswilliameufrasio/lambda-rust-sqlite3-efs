use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Sqlite>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CreateUserResponse {
    pub id: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueuedUser {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct MultipleUsersResult {
    pub users: Vec<User>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SqsRecord {
    #[serde(rename = "messageId")]
    pub message_id: Option<String>,
    #[serde(rename = "receiptHandle")]
    pub receipt_handle: Option<String>,
    pub body: Option<String>,
    pub attributes: Option<serde_json::Value>,
    #[serde(rename = "messageAttributes")]
    pub message_attributes: Option<serde_json::Value>,
    #[serde(rename = "md5OfBody")]
    pub md5_of_body: Option<String>,
    #[serde(rename = "eventSource")]
    pub event_source: Option<String>,
    #[serde(rename = "eventSourceARN")]
    pub event_source_arn: Option<String>,
    #[serde(rename = "awsRegion")]
    pub aws_region: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SqsEvent {
    #[serde(rename = "Records")]
    pub records: Vec<SqsRecord>,
}

impl QueuedUser {
    pub fn from_create_request(req: &CreateUserRequest, id: String) -> Self {
        QueuedUser {
            id,
            name: req.name.clone(),
            email: req.email.clone(),
        }
    }
}
