pub mod render_xml;

use crate::render_xml::RenderXml;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_template::{engine::Engine, RenderHtml};
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

#[derive(Serialize, Deserialize)]
pub struct StopSchedule {
    stop: Stop,
}

#[derive(Serialize, Deserialize)]
pub struct Stop {
    name: String,
}

#[derive(Serialize)]
pub struct TimesResponse {
    stop_schedule: StopSchedule,
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

        RenderXml(
            "times",
            state.engine,
            TimesResponse {
                stop_schedule: response.stop_schedule,
            },
        )
        .into_response()
    } else {
        RenderXml("twilio", state.engine, ()).into_response()
    }
}
