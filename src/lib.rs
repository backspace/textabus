pub mod config;
pub mod render_xml;
pub mod routes;

use crate::config::{Config, ConfigProvider, EnvVarProvider};
use crate::routes::*;

use axum::{routing::get, Router};
use axum_template::engine::Engine;
use handlebars::{DirectorySourceOptions, Handlebars};
use std::env;

type AppEngine = Engine<Handlebars<'static>>;

#[derive(Clone)]
pub struct AppState {
    config: Config,
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

    let env_config_provider = EnvVarProvider::new(env::vars().collect());
    let config = env_config_provider.get_config();

    Router::new()
        .route("/", get(get_root))
        .route("/twilio", get(get_twilio))
        .with_state(AppState {
            config: config.clone(),
            engine: Engine::from(hbs),
            winnipeg_transit_api_address: services.winnipeg_transit_api_address.unwrap(),
        })
}
