use crate::AppState;

use axum::{extract::State, response::IntoResponse};
use axum_template::RenderHtml;

pub async fn get_root(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("root", state.engine, ())
}

pub async fn get_about(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("about", state.engine, ())
}

pub async fn get_changelog(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("changelog", state.engine, ())
}
