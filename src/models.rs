use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::types::uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct ApiResponse {
    pub id: Uuid,
    pub body: String,
    pub query: String,
    pub message_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, sqlx::FromRow, Serialize)]
pub struct Message {
    pub id: Uuid,
    pub message_sid: Option<String>,
    pub origin: String,
    pub destination: String,
    pub body: String,
    pub initial_message_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Number {
    pub number: String,
    pub name: Option<String>,
    pub approved: bool,
    pub admin: bool,
}
