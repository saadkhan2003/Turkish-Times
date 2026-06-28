use axum::{extract::{Path, State}, response::{Html, Redirect}, Form};
use std::sync::Arc;
use serde::Deserialize;
use crate::AppState;
use crate::models::Season;

pub async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let seasons = sqlx::query_as::<_, Season>(
        "SELECT * FROM seasons ORDER BY created_at DESC"
    ).fetch_all(&state.db).await.unwrap_or_default();
    let ctx = tera::Context::from_serialize(serde_json::json!({"seasons": seasons, "app_url": &state.config.app_url})).unwrap();
    let body = state.tera.render("admin/seasons/index.html", &ctx).unwrap_or_else(|e| format!("Error: {}", e));
    Html(body)
}

pub async fn create(State(state): State<Arc<AppState>>) -> Html<String> {
    let all_series = sqlx::query_as::<_, crate::models::Series>("SELECT * FROM series ORDER BY title")
        .fetch_all(&state.db).await.unwrap_or_default();
    let ctx = tera::Context::from_serialize(serde_json::json!({
        "all_series": all_series, "app_url": &state.config.app_url,
    })).unwrap();
    let body = state.tera.render("admin/seasons/create.html", &ctx).unwrap_or_else(|e| format!("Error: {}", e));
    Html(body)
}

#[derive(Deserialize)]
pub struct SeasonForm {
    series_id: i64,
    season_number: i64,
    title: Option<String>,
}

pub async fn store(State(state): State<Arc<AppState>>, Form(f): Form<SeasonForm>) -> Redirect {
    sqlx::query("INSERT INTO seasons (series_id, season_number, title, created_at, updated_at) VALUES (?, ?, ?, datetime('now'), datetime('now'))")
        .bind(f.series_id).bind(f.season_number).bind(&f.title)
        .execute(&state.db).await.unwrap_or_default();
    Redirect::to("/admin/seasons")
}

pub async fn edit(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Html<String> {
    let s = sqlx::query_as::<_, (i64,i64,i64,Option<String>,Option<String>,Option<String>,Option<i64>,Option<String>,Option<String>)>(
        "SELECT * FROM seasons WHERE id = ?"
    ).bind(id).fetch_optional(&state.db).await.unwrap_or(None);
    match s {
        Some(season) => Html(format!("<h1>Edit Season</h1><form method='POST'><input name='title' value='{}'><input name='season_number' value='{}'><button>Update</button></form>",
            season.3.as_deref().unwrap_or(""), season.2)),
        None => Html("<h1>Not Found</h1>".to_string()),
    }
}

pub async fn update(State(state): State<Arc<AppState>>, Path(id): Path<i64>, Form(f): Form<SeasonForm>) -> Redirect {
    sqlx::query("UPDATE seasons SET season_number=?, title=?, updated_at=datetime('now') WHERE id=?")
        .bind(f.season_number).bind(&f.title).bind(id)
        .execute(&state.db).await.unwrap_or_default();
    Redirect::to("/admin/seasons")
}

pub async fn destroy(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Redirect {
    sqlx::query("DELETE FROM episodes WHERE season_id = ?").bind(id).execute(&state.db).await.unwrap_or_default();
    sqlx::query("DELETE FROM seasons WHERE id = ?").bind(id).execute(&state.db).await.unwrap_or_default();
    Redirect::to("/admin/seasons")
}
