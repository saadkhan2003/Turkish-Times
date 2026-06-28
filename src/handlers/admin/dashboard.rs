use axum::{extract::State, response::{Html, IntoResponse, Redirect}};
use std::sync::Arc;
use crate::AppState;

pub async fn dashboard(State(state): State<Arc<AppState>>) -> Html<String> {
    let series_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM series")
        .fetch_one(&state.db).await.unwrap_or(0);
    let seasons_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM seasons")
        .fetch_one(&state.db).await.unwrap_or(0);
    let episodes_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM episodes")
        .fetch_one(&state.db).await.unwrap_or(0);
    let subtitles_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM subtitles")
        .fetch_one(&state.db).await.unwrap_or(0);

    let ctx = tera::Context::from_serialize(serde_json::json!({
        "series_count": series_count, "seasons_count": seasons_count,
        "episodes_count": episodes_count, "subtitles_count": subtitles_count,
        "app_url": &state.config.app_url,
    })).unwrap();

    let body = state.tera.render("admin/dashboard.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {}", e));
    Html(body)
}
