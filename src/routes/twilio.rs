use crate::{models::Number, render_xml::RenderXml, AppState};

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

const MAX_RESPONSE_LENGTH: usize = 140;
const DELAY_THRESHOLD: i64 = 3;
const AHEAD_THRESHOLD: i64 = 1;

#[axum_macros::debug_handler]
pub async fn get_twilio(
    State(state): State<AppState>,
    params: Query<TwilioParams>,
) -> impl IntoResponse {
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
                let maybe_stop_and_routes = parse_stop_and_routes(&body);

                if maybe_stop_and_routes.is_ok() {
                    let (stop_number, routes) = maybe_stop_and_routes.unwrap();

                    let client = reqwest::Client::new();

                    let query = format!("/v3/stops/{}/schedule.json?usage=short", stop_number,);

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
                        parsed_response.stop_schedule.stop.number,
                        parsed_response.stop_schedule.stop.name
                    );

                    let mut schedule_lines: Vec<(NaiveDateTime, String)> = Vec::new();

                    for route_schedule in &parsed_response.stop_schedule.route_schedules {
                        let number_as_string;
                        let route_number = match &route_schedule.route.number {
                            Value::String(s) => s,
                            Value::Number(n) => {
                                number_as_string = n.to_string();
                                &number_as_string
                            }
                            _ => panic!("Unexpected type parsing route number"),
                        };

                        if !routes.is_empty() && !routes.contains(&route_number.as_str()) {
                            continue;
                        }

                        for scheduled_stop in &route_schedule.scheduled_stops {
                            let time = NaiveDateTime::parse_from_str(
                                &scheduled_stop.times.departure.estimated,
                                "%Y-%m-%dT%H:%M:%S",
                            )
                            .unwrap();

                            let scheduled_time = NaiveDateTime::parse_from_str(
                                &scheduled_stop.times.departure.scheduled,
                                "%Y-%m-%dT%H:%M:%S",
                            )
                            .unwrap();

                            let mut line =
                                format!("{} {}", route_number, scheduled_stop.variant.name);

                            if time.signed_duration_since(scheduled_time).num_minutes()
                                >= DELAY_THRESHOLD
                            {
                                line.push_str(
                                    format!(
                                        " ({}min delay)",
                                        time.signed_duration_since(scheduled_time).num_minutes()
                                    )
                                    .as_str(),
                                );
                            } else if time.signed_duration_since(scheduled_time).num_minutes()
                                <= -AHEAD_THRESHOLD
                            {
                                line.push_str(
                                    format!(
                                        " ({}min ahead)",
                                        time.signed_duration_since(scheduled_time)
                                            .num_minutes()
                                            .abs()
                                    )
                                    .as_str(),
                                );
                            }

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

                    for line in sorted_schedule_lines {
                        if response_text.len() + line.len() < MAX_RESPONSE_LENGTH {
                            response_text.push_str(&format!("{}\n", line));
                        } else {
                            break;
                        }
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
    scheduled: String,
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
