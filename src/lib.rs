pub mod auth;
pub mod commands;
pub mod config;
pub mod models;
pub mod odws;
pub mod render_xml;
pub mod routes;

use crate::config::{Config, ConfigProvider, EnvVarProvider};
use crate::routes::*;

use axum::{
    routing::{get, post},
    Router,
};
use axum_template::engine::Engine;
use handlebars::{DirectorySourceOptions, Handlebars};
use sqlx::postgres::PgPool;
use std::env;
use tower_http::services::ServeDir;

type AppEngine = Engine<Handlebars<'static>>;

#[derive(Clone)]
pub struct AppState {
    config: Config,
    db: PgPool,
    engine: AppEngine,
    twilio_address: String,
    winnipeg_transit_api_address: String,
}

pub struct InjectableServices {
    pub db: PgPool,
    pub twilio_address: Option<String>,
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
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get(get_root))
        .route("/about", get(get_about))
        .route("/changelog", get(get_changelog))
        .route("/twilio", get(get_twilio))
        .route("/raw", get(get_raw))
        .route("/admin/messages", get(get_messages))
        .route("/admin/numbers", get(get_numbers))
        .route("/admin/numbers/:number/approve", post(post_approve_number))
        .route(
            "/admin/numbers/:number/unapprove",
            post(post_unapprove_number),
        )
        .with_state(AppState {
            config: config.clone(),
            db: services.db,
            engine: Engine::from(hbs),
            twilio_address: services.twilio_address.unwrap(),
            winnipeg_transit_api_address: services.winnipeg_transit_api_address.unwrap(),
        })
}
