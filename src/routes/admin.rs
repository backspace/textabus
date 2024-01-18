use crate::{auth::User, models::Number, AppState};

use axum::{extract::State, response::IntoResponse};
use axum_template::RenderHtml;
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::types::uuid::Uuid;
use std::collections::HashMap;

pub async fn get_messages(State(state): State<AppState>, _user: User) -> impl IntoResponse {
    let messages = sqlx::query_as::<_, ExtendedMessage>(
        r#"
            SELECT messages.*, to_char(messages.created_at, 'Mon DD, YYYY HH24:MI') AS formatted_created_at, numbers.name AS origin_name
            FROM messages
            LEFT JOIN numbers ON messages.origin = numbers.number
            ORDER BY messages.created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .expect("Failed to fetch messages");

    let mut exchanges: Vec<Exchange> = Vec::new();
    let mut replies: HashMap<Uuid, Vec<ExtendedMessage>> = HashMap::new();

    for message in messages {
        if let Some(initial_message_id) = message.initial_message_id {
            replies
                .entry(initial_message_id)
                .or_default()
                .insert(0, message);
            // Prepend because the replies will be in reverse order
        } else {
            exchanges.push(Exchange {
                first: message.clone(),
                responses: Vec::new(),
            });
        }
    }

    for exchange in &mut exchanges {
        if let Some(reply_messages) = replies.remove(&exchange.first.id) {
            exchange.responses = reply_messages;
        }
    }

    RenderHtml(
        "admin/messages",
        state.engine,
        MessagesTemplate { exchanges },
    )
}

pub async fn get_numbers(State(state): State<AppState>, _user: User) -> impl IntoResponse {
    let numbers = sqlx::query_as::<_, Number>(
        r#"
            SELECT *
            FROM numbers
            ORDER BY created_at desc
        "#,
    )
    .fetch_all(&state.db)
    .await
    .expect("Failed to fetch messages");

    let (approved, unapproved): (Vec<Number>, Vec<Number>) =
        numbers.into_iter().partition(|number| number.approved);

    RenderHtml(
        "admin/numbers",
        state.engine,
        NumbersTemplate {
            unapproved,
            approved,
        },
    )
}

#[derive(Serialize)]
struct MessagesTemplate {
    exchanges: Vec<Exchange>,
}

#[derive(Debug, Serialize)]
struct Exchange {
    first: ExtendedMessage,
    responses: Vec<ExtendedMessage>,
}

#[derive(Clone, Debug, sqlx::FromRow, Serialize)]
pub struct ExtendedMessage {
    pub id: Uuid,
    pub message_sid: Option<String>,
    pub origin: String,
    pub origin_name: Option<String>,
    pub destination: String,
    pub body: String,
    pub initial_message_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub formatted_created_at: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize)]
struct NumbersTemplate {
    unapproved: Vec<Number>,
    approved: Vec<Number>,
}
