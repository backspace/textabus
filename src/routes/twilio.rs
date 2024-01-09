use crate::{render_xml::RenderXml, AppState};

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::{NaiveDateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, NoneAsEmptyString};
use sqlx::types::Uuid;

#[axum_macros::debug_handler]
pub async fn get_twilio(
    State(state): State<AppState>,
    params: Query<TwilioParams>,
) -> impl IntoResponse {
    let stop_number_regex = Regex::new(r"^\d{5}$").unwrap();

    let api_key = &state.config.winnipeg_transit_api_key;

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

    if params.body.is_some() && stop_number_regex.is_match(&params.body.clone().unwrap()) {
        let client = reqwest::Client::new();

        let query = format!(
            "/v3/stops/{}/schedule.json?usage=short",
            params.body.clone().unwrap(),
        );

        let api_response_text = client
            .get(format!(
                "{}{}&api-key={}",
                state.winnipeg_transit_api_address,
                query.clone(),
                api_key
            ))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let api_response_insertion_result = sqlx::query(
            r#"
            INSERT INTO api_responses (id, body, query, message_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(api_response_text.clone())
        .bind(query)
        .bind(maybe_incoming_message_id)
        .bind(Utc::now().naive_utc())
        .bind(Utc::now().naive_utc())
        .execute(&state.db)
        .await;

        if let Err(e) = api_response_insertion_result {
            log::error!("Failed to insert API response: {}", e);
        }

        let parsed_response =
            serde_json::from_str::<StopScheduleResponse>(&api_response_text).unwrap();

        response_text = format!(
            "{} {}\n",
            parsed_response.stop_schedule.stop.number, parsed_response.stop_schedule.stop.name
        );

        let mut schedule_lines: Vec<(NaiveDateTime, String)> = Vec::new();

        for route_schedule in &parsed_response.stop_schedule.route_schedules {
            for scheduled_stop in &route_schedule.scheduled_stops {
                let time = NaiveDateTime::parse_from_str(
                    &scheduled_stop.times.departure.estimated,
                    "%Y-%m-%dT%H:%M:%S",
                )
                .unwrap();

                let route_number = match route_schedule.route.number.clone() {
                    Value::String(s) => s,
                    Value::Number(n) => n.to_string(),
                    _ => panic!("Unexpected type for number"),
                };

                let line = format!("{} {}", route_number, scheduled_stop.variant.name);
                schedule_lines.push((time, line));
            }
        }

        schedule_lines.sort_by(|a, b| a.0.cmp(&b.0));

        let sorted_schedule_lines: Vec<String> = schedule_lines
            .iter()
            .map(|(time, line)| {
                format!(
                    "{} {}",
                    time.format("%-I:%M%p")
                        .to_string()
                        .to_lowercase()
                        .trim_end_matches('m'),
                    line
                )
            })
            .collect();

        const MAX_RESPONSE_LENGTH: usize = 140;

        for line in sorted_schedule_lines {
            if response_text.len() + line.len() < MAX_RESPONSE_LENGTH {
                response_text.push_str(&format!("{}\n", line));
            } else {
                break;
            }
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

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct StopScheduleResponse {
    stop_schedule: StopSchedule,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StopSchedule {
    stop: Stop,
    route_schedules: Vec<RouteSchedule>,
}

#[derive(Deserialize)]
pub struct Stop {
    name: String,
    number: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RouteSchedule {
    route: Route,
    scheduled_stops: Vec<ScheduledStop>,
}

#[derive(Deserialize)]
struct ScheduledStop {
    times: Times,
    variant: Variant,
}

#[derive(Deserialize)]
struct Times {
    departure: ArrivalDeparture,
}

#[derive(Deserialize)]
struct ArrivalDeparture {
    estimated: String,
}

#[derive(Deserialize)]
struct Route {
    number: Value,
}

#[derive(Deserialize)]
struct Variant {
    name: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    body: String,
}
