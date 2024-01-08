use crate::AppState;

use axum::{extract::State, response::IntoResponse};
use axum_template::RenderHtml;

pub async fn get_root(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("root", state.engine, ())
}
