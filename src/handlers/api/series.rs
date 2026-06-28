use axum::{extract::{Path, State}, response::Json};
use std::sync::Arc;
use serde_json::{json, Value};
use crate::AppState;
use crate::models::{Series, Season, Episode};

pub async fn index(State(state): State<Arc<AppState>>) -> Json<Value> {
    let series = sqlx::query_as::<_, Series>(
        "SELECT * FROM series ORDER BY created_at DESC LIMIT 20"
    ).fetch_all(&state.db).await.unwrap_or_default();
    Json(json!(series))
}

pub async fn show(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Json<Value> {
    let series = sqlx::query_as::<_, Series>("SELECT * FROM series WHERE id = ?")
        .bind(id).fetch_optional(&state.db).await.unwrap_or(None);
    match series {
        Some(s) => Json(json!(s)),
        None => Json(json!({"error": "Not found"})),
    }
}

pub async fn seasons(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Json<Value> {
    let seasons = sqlx::query_as::<_, Season>(
        "SELECT * FROM seasons WHERE series_id = ? ORDER BY season_number"
    ).bind(id).fetch_all(&state.db).await.unwrap_or_default();
    Json(json!(seasons))
}

pub async fn episodes(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Json<Value> {
    let episodes = sqlx::query_as::<_, Episode>(
        "SELECT * FROM episodes WHERE season_id = ? ORDER BY episode_number"
    ).bind(id).fetch_all(&state.db).await.unwrap_or_default();
    Json(json!(episodes))
}

pub async fn episode_detail(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Json<Value> {
    let episode = sqlx::query_as::<_, Episode>("SELECT * FROM episodes WHERE id = ?")
        .bind(id).fetch_optional(&state.db).await.unwrap_or(None);
    match episode {
        Some(ep) => {
            sqlx::query("UPDATE episodes SET views = COALESCE(views, 0) + 1 WHERE id = ?")
                .bind(ep.id).execute(&state.db).await.unwrap_or_default();
            Json(json!(ep))
        }
        None => Json(json!({"error": "Not found"})),
    }
}
