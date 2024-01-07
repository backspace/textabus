pub mod render_xml;

use crate::render_xml::RenderXml;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_template::{engine::Engine, RenderHtml};
use chrono::NaiveDateTime;
use handlebars::{DirectorySourceOptions, Handlebars};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};

type AppEngine = Engine<Handlebars<'static>>;

#[derive(Clone)]
pub struct AppState {
    engine: AppEngine,
    winnipeg_transit_api_address: String,
}

pub struct InjectableServices {
    pub winnipeg_transit_api_address: Option<String>,
}

pub async fn app(services: InjectableServices) -> Router {
    let mut hbs = Handlebars::new();
    hbs.register_templates_directory(
        "templates",
        DirectorySourceOptions {
            tpl_extension: ".hbs".to_string(),
            hidden: false,
            temporary: false,
        },
    )
    .expect("Failed to register templates directory");

    Router::new()
        .route("/", get(get_root))
        .route("/twilio", get(get_twilio))
        .with_state(AppState {
            engine: Engine::from(hbs),
            winnipeg_transit_api_address: services.winnipeg_transit_api_address.unwrap(),
        })
}

async fn get_root(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("root", state.engine, ())
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
    number: u32,
}

#[derive(Deserialize)]
struct Variant {
    name: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    body: String,
}

#[axum_macros::debug_handler]
async fn get_twilio(
    State(state): State<AppState>,
    params: Query<TwilioParams>,
) -> impl IntoResponse {
    let stop_number_regex = Regex::new(r"^\d{5}$").unwrap();

    let api_key = "FIXME";

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

        for route_schedule in &response.stop_schedule.route_schedules {
            for scheduled_stop in &route_schedule.scheduled_stops {
                let time = NaiveDateTime::parse_from_str(
                    &scheduled_stop.times.departure.estimated,
                    "%Y-%m-%dT%H:%M:%S",
                )
                .unwrap()
                .format("%-I:%M%p")
                .to_string()
                .to_lowercase()
                .trim_end_matches('m')
                .to_string();
                response_text.push_str(&format!(
                    "{} {} {}\n",
                    time, route_schedule.route.number, scheduled_stop.variant.name
                ));
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
