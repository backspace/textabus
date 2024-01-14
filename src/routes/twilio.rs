use crate::{
    commands::{handle_stops_request, handle_times_request, parse_command, Command},
    models::Number,
    render_xml::RenderXml,
    AppState,
};

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use sqlx::types::Uuid;

#[axum_macros::debug_handler]
pub async fn get_twilio(
    State(state): State<AppState>,
    params: Query<TwilioParams>,
) -> impl IntoResponse {
    let incoming_message_id = Uuid::new_v4();
    let incoming_message_insertion_result = sqlx::query(
        r#"
        INSERT INTO messages (id, message_sid, origin, destination, body, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(incoming_message_id)
    .bind(params.message_sid.clone())
    .bind(params.from.clone())
    .bind(params.to.clone())
    .bind(params.body.clone())
    .bind(Utc::now().naive_utc())
    .bind(Utc::now().naive_utc())
    .execute(&state.db)
    .await;

    let mut maybe_incoming_message_id = Some(incoming_message_id);

    if let Err(e) = incoming_message_insertion_result {
        maybe_incoming_message_id = None;
        log::error!("Failed to insert incoming message: {}", e);
    }

    let mut response_text = "textabus".to_string();

    let number = sqlx::query_as::<_, Number>(
        r#"
        SELECT * FROM numbers
        WHERE number = $1
        "#,
    )
    .bind(params.from.clone())
    .fetch_one(&state.db)
    .await;

    if number.is_ok() {
        if number.unwrap().approved {
            if params.body.is_some() {
                let body = params.body.clone().unwrap();

                let command = parse_command(&body);

                response_text = match command {
                    Command::Stops(stops_command) => handle_stops_request(
                        stops_command,
                        &state.config,
                        state.winnipeg_transit_api_address.clone(),
                        maybe_incoming_message_id,
                        &state.db,
                    )
                    .await
                    .unwrap(),
                    Command::Times(times_command) => handle_times_request(
                        times_command,
                        &state.config,
                        state.winnipeg_transit_api_address.clone(),
                        maybe_incoming_message_id,
                        &state.db,
                    )
                    .await
                    .unwrap(),
                    Command::Unknown(_unknown_command) => "textabus".to_string(),
                };
            }
        } else {
            return (axum::http::StatusCode::NOT_FOUND, "not found").into_response();
        }
    } else {
        response_text = "welcome to textabus. we don’t recognise you, please contact a maintainer to join the alpha test.".to_string();

        let number_insertion_result = sqlx::query(
            r#"
            INSERT INTO numbers (number, created_at, updated_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(params.from.clone())
        .bind(Utc::now().naive_utc())
        .bind(Utc::now().naive_utc())
        .execute(&state.db)
        .await;

        if let Err(e) = number_insertion_result {
            log::error!("Failed to insert number: {}", e);
        }
    }

    let outgoing_message_insertion_result = sqlx::query(
        r#"
        INSERT INTO messages (id, origin, destination, body, initial_message_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(params.to.clone())
    .bind(params.from.clone())
    .bind(response_text.clone())
    .bind(maybe_incoming_message_id)
    .bind(Utc::now().naive_utc())
    .bind(Utc::now().naive_utc())
    .execute(&state.db)
    .await;

    if let Err(e) = outgoing_message_insertion_result {
        log::error!("Failed to insert outgoing message: {}", e);
    }

    RenderXml(
        "message-response",
        state.engine,
        MessageResponse {
            body: response_text,
        },
    )
    .into_response()
}

#[serde_as]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TwilioParams {
    #[serde_as(as = "NoneAsEmptyString")]
    pub body: Option<String>,
    pub message_sid: String,
    pub from: String,
    pub to: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    body: String,
}
