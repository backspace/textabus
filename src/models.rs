use chrono::NaiveDateTime;
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

#[derive(Debug, sqlx::FromRow)]
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
