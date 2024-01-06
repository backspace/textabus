use axum::{extract::State, response::IntoResponse, routing::get, Router};
use axum_template::{engine::Engine, RenderHtml};
use handlebars::{DirectorySourceOptions, Handlebars};

type AppEngine = Engine<Handlebars<'static>>;

#[derive(Clone)]
pub struct AppState {
    engine: AppEngine,
}

pub async fn app() -> Router {
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
        .with_state(AppState {
            engine: Engine::from(hbs),
        })
}

async fn get_root(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("root", state.engine, ())
}
