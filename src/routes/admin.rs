use crate::{auth::User, models::Number, routes::HELP_MESSAGE, AppState};

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use axum_template::RenderHtml;
use base64::{engine::general_purpose, Engine as _};
use chrono::{NaiveDateTime, Utc};
use http::StatusCode;
use serde::Serialize;
use sqlx::types::uuid::Uuid;
use std::collections::HashMap;

pub const APPROVAL_MESSAGE: &str = "you have been approved to beta test textabus!\n\nmessages are stored for debugging. please let admin know if you find a bug or have suggestions for improvement";

pub fn get_composed_approval_message() -> String {
    format!("{}\n\n{}", APPROVAL_MESSAGE, HELP_MESSAGE)
}

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

#[axum_macros::debug_handler]
pub async fn post_approve_number(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let config = state.config;

    let account_sid = config.twilio_account_sid.to_string();
    let api_sid = config.twilio_api_key_sid.to_string();
    let api_secret = config.twilio_api_key_secret.to_string();

    let client = reqwest::Client::new();

    let basic_auth = format!("{}:{}", api_sid, api_secret);
    let auth_header_value = format!(
        "Basic {}",
        general_purpose::STANDARD_NO_PAD.encode(basic_auth)
    );

    let approved_notification_body = get_composed_approval_message();

    sqlx::query(
        r#"
            UPDATE numbers
            SET approved = TRUE
            WHERE number = $1
        "#,
    )
    .bind(id.clone())
    .execute(&state.db)
    .await
    .expect("Failed to update number");

    let create_message_body = serde_urlencoded::to_string([
        ("Body", approved_notification_body.clone()),
        ("To", id.clone()),
        ("From", config.textabus_number.clone()),
    ])
    .expect("Could not encode meeting message creation body");

    client
        .post(format!(
            "{}/2010-04-01/Accounts/{}/Messages.json",
            state.twilio_address, account_sid
        ))
        .header("Authorization", auth_header_value.clone())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(create_message_body.clone())
        .send()
        .await
        .ok();

    let admin_message_insertion_result = sqlx::query(
        r#"
        INSERT INTO messages (id, origin, destination, body, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(config.textabus_number)
    .bind(id)
    .bind(approved_notification_body)
    .bind(Utc::now().naive_utc())
    .bind(Utc::now().naive_utc())
    .execute(&state.db)
    .await;

    if let Err(e) = admin_message_insertion_result {
        log::error!("Failed to insert approval message: {}", e);
    }

    StatusCode::NO_CONTENT.into_response()
}

#[axum_macros::debug_handler]
pub async fn post_unapprove_number(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    sqlx::query(
        r#"
            UPDATE numbers
            SET approved = FALSE
            WHERE number = $1
        "#,
    )
    .bind(id)
    .execute(&state.db)
    .await
    .expect("Failed to update number");

    StatusCode::NO_CONTENT.into_response()
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
