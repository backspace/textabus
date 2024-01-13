use crate::{
    commands::{handle_stops_request, handle_times_request},
    models::Number,
    render_xml::RenderXml,
    AppState,
};

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::Utc;
use regex::Regex;
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

                let maybe_stops_and_location = parse_stops_and_location(&body);

                if maybe_stops_and_location.is_ok() {
                    response_text = handle_stops_request(
                        &state.config,
                        state.winnipeg_transit_api_address.clone(),
                        maybe_stops_and_location.unwrap(),
                        maybe_incoming_message_id,
                        &state.db,
                    )
                    .await
                    .unwrap();
                } else {
                    let maybe_stop_and_routes = parse_stop_and_routes(&body);

                    if maybe_stop_and_routes.is_ok() {
                        let (stop_number, routes) = maybe_stop_and_routes.unwrap();

                        response_text = handle_times_request(
                            &state.config,
                            state.winnipeg_transit_api_address.clone(),
                            stop_number,
                            routes,
                            maybe_incoming_message_id,
                            &state.db,
                        )
                        .await
                        .unwrap();
                    }
                }
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

fn parse_stop_and_routes(input: &str) -> Result<(&str, Vec<&str>), &'static str> {
    let re = Regex::new(r"^(\d{5})(?:\s+(.*))?$").unwrap();

    if let Some(captures) = re.captures(input) {
        let stop_number = captures.get(1).map_or("", |m| m.as_str());
        let routes: Vec<&str> = captures
            .get(2)
            .map_or("", |m| m.as_str())
            .split_whitespace()
            .collect();
        Ok((stop_number, routes))
    } else {
        Err("Input string doesn't match the expected pattern")
    }
}

fn parse_stops_and_location(input: &str) -> Result<&str, &'static str> {
    let re = Regex::new(r"^stops\s+(.*)$").unwrap();

    if let Some(captures) = re.captures(input) {
        let location = captures.get(1).map_or("", |m| m.as_str());
        Ok(location)
    } else {
        Err("Input string does not match a stops request")
    }
}
