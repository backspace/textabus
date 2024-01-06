pub mod render_xml;

use crate::render_xml::RenderXml;
use axum::{extract::State, response::IntoResponse, routing::get, Router};
use axum_template::{engine::Engine, RenderHtml};
use handlebars::{DirectorySourceOptions, Handlebars};

type AppEngine = Engine<Handlebars<'static>>;

#[derive(Clone)]
pub struct AppState {
    engine: AppEngine,
}

pub struct InjectableServices {}

pub async fn app(_services: InjectableServices) -> Router {
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
        })
}

async fn get_root(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("root", state.engine, ())
}

async fn get_twilio(State(state): State<AppState>) -> impl IntoResponse {
    RenderXml("twilio", state.engine, ())
}
