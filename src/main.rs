use axum::{extract::State, response::IntoResponse, routing::get, Router};
use axum_template::{engine::Engine, RenderHtml};
use handlebars::{DirectorySourceOptions, Handlebars};

type AppEngine = Engine<Handlebars<'static>>;

#[derive(Clone)]
struct AppState {
    engine: AppEngine,
}

#[tokio::main]
async fn main() {
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

    let app = Router::new()
        .route("/", get(get_root))
        .with_state(AppState {
            engine: Engine::from(hbs),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:1312").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_root(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("root", state.engine, ())
}
