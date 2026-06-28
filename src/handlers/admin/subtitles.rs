use axum::{extract::{State}, response::{Html, Redirect}, Form};
use std::sync::Arc;
use serde::Deserialize;
use crate::AppState;
use crate::models::Subtitle;

pub async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let subtitles = sqlx::query_as::<_, Subtitle>("SELECT * FROM subtitles ORDER BY created_at DESC")
        .fetch_all(&state.db).await.unwrap_or_default();
    let ctx = tera::Context::from_serialize(serde_json::json!({
        "subtitles": subtitles, "app_url": &state.config.app_url,
    })).unwrap();
    let body = state.tera.render("admin/subtitles/index.html", &ctx)
        .unwrap_or_else(|e| format!("Error: {}", e));
    Html(body)
}

pub async fn create(State(state): State<Arc<AppState>>) -> Html<String> {
    let ctx = tera::Context::from_serialize(serde_json::json!({"app_url": &state.config.app_url})).unwrap();
    let body = state.tera.render("admin/subtitles/create.html", &ctx).unwrap_or_else(|e| format!("Error: {}", e));
    Html(body)
}
