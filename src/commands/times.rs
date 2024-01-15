use chrono::{NaiveDateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use sqlx::{types::Uuid, PgPool};

use crate::{commands::TimesCommand, config::Config};

const MAX_RESPONSE_LENGTH: usize = 140;
const DELAY_THRESHOLD: i64 = 3;
const AHEAD_THRESHOLD: i64 = 1;

pub async fn handle_times_request(
    command: TimesCommand,
    config: &Config,
    winnipeg_transit_api_address: String,
    maybe_incoming_message_id: Option<Uuid>,
    db: &PgPool,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_key = config.winnipeg_transit_api_key.clone();

    let query = format!(
        "/v3/stops/{}/schedule.json?usage=short",
        command.stop_number,
    );

    let api_response = client
        .get(format!(
            "{}{}&api-key={}",
            winnipeg_transit_api_address,
            query.clone(),
            api_key
        ))
        .send()
        .await
        .unwrap();

    let success = api_response.status().is_success();

    let api_response_text = api_response.text().await.unwrap();

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
    .execute(db)
    .await;

    if let Err(e) = api_response_insertion_result {
        log::error!("Failed to insert API response: {}", e);
    }

    if !success {
        return Ok(format!(
            "No schedule found for stop {}, does it exist?",
            command.stop_number
        ));
    }

    let parsed_response = serde_json::from_str::<StopScheduleResponse>(&api_response_text).unwrap();

    let mut response_text = format!(
        "{} {}\n",
        parsed_response.stop_schedule.stop.number, parsed_response.stop_schedule.stop.name
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

        if !command.routes.is_empty() && !command.routes.contains(route_number) {
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

            let mut line = format!("{} {}", route_number, scheduled_stop.variant.name);

            if time.signed_duration_since(scheduled_time).num_minutes() >= DELAY_THRESHOLD {
                line.push_str(
                    format!(
                        " ({}min delay)",
                        time.signed_duration_since(scheduled_time).num_minutes()
                    )
                    .as_str(),
                );
            } else if time.signed_duration_since(scheduled_time).num_minutes() <= -AHEAD_THRESHOLD {
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

    Ok(response_text)
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
