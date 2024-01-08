use crate::{render_xml::RenderXml, AppState};

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::NaiveDateTime;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, NoneAsEmptyString};

#[axum_macros::debug_handler]
pub async fn get_twilio(
    State(state): State<AppState>,
    params: Query<TwilioParams>,
) -> impl IntoResponse {
    let stop_number_regex = Regex::new(r"^\d{5}$").unwrap();

    let api_key = &state.config.winnipeg_transit_api_key;

    if params.body.is_some() && stop_number_regex.is_match(&params.body.clone().unwrap()) {
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "{}/v3/stops/{}/schedule.json?usage=short&api-key={}",
                state.winnipeg_transit_api_address,
                params.body.clone().unwrap(),
                api_key
            ))
            .send()
            .await
            .unwrap()
            .json::<StopScheduleResponse>()
            .await
            .unwrap();

        let mut response_text = format!(
            "{} {}\n",
            response.stop_schedule.stop.number, response.stop_schedule.stop.name
        );

        let mut schedule_lines: Vec<(NaiveDateTime, String)> = Vec::new();

        for route_schedule in &response.stop_schedule.route_schedules {
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
        RenderXml(
            "message-response",
            state.engine,
            MessageResponse {
                body: response_text,
            },
        )
        .into_response()
    } else {
        RenderXml("twilio", state.engine, ()).into_response()
    }
}

#[serde_as]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TwilioParams {
    #[serde_as(as = "NoneAsEmptyString")]
    pub body: Option<String>,
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
