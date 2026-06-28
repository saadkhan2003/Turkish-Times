use axum::extract::{Form, Multipart, Path, State};
use axum::response::{Html, Redirect};
use serde::Deserialize;
use std::sync::Arc;
use std::str::FromStr;
use crate::AppState;

#[derive(Deserialize, Default)]
pub struct EpisodeForm {
    pub season_id: Option<i64>,
    pub episode_number: Option<i64>,
    pub title: Option<String>,
    pub video_source_primary: Option<String>,
    pub duration: Option<i64>,
}
use crate::models::{Episode, Series, Season};

pub async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let episodes = sqlx::query_as::<_, Episode>(
        "SELECT * FROM episodes ORDER BY created_at DESC LIMIT 50"
    ).fetch_all(&state.db).await.unwrap_or_default();
    let ctx = tera::Context::from_serialize(serde_json::json!({
        "episodes": episodes, "app_url": &state.config.app_url,
    })).unwrap();
    let body = state.tera.render("admin/episodes/index.html", &ctx).unwrap_or_else(|e| format!("Error: {}", e));
    Html(body)
}

pub async fn create(State(state): State<Arc<AppState>>) -> Html<String> {
    let all_series = sqlx::query_as::<_, Series>("SELECT * FROM series ORDER BY title")
        .fetch_all(&state.db).await.unwrap_or_default();
    let all_seasons = sqlx::query_as::<_, Season>("SELECT * FROM seasons ORDER BY series_id, season_number")
        .fetch_all(&state.db).await.unwrap_or_default();

    let seasons_json = serde_json::to_string(&all_seasons).unwrap_or_else(|_| "[]".to_string());
    let ctx = tera::Context::from_serialize(serde_json::json!({
        "all_series": all_series, "all_seasons": all_seasons,
        "seasons_json": seasons_json, "app_url": &state.config.app_url,
    })).unwrap();
    let body = state.tera.render("admin/episodes/create.html", &ctx).unwrap_or_else(|e| format!("Error: {}", e));
    Html(body)
}

pub async fn store(State(state): State<Arc<AppState>>, mut multipart: axum::extract::Multipart) -> Redirect {
    let mut season_id = None;
    let mut episode_number = None;
    let mut title = None;
    let mut description = None;
    let mut video_source_primary = None;
    let mut video_source_backup = None;
    let mut duration = None;
    let mut download_link = None;
    let mut size_1080p = None;
    let mut download_bluray = None;
    let mut size_bluray = None;
    let mut download_720p = None;
    let mut size_720p = None;
    let mut download_480p = None;
    let mut size_480p = None;
    let mut thumbnail_name = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "season_id" => season_id = field.text().await.ok().and_then(|v| i64::from_str(&v).ok()),
            "episode_number" => episode_number = field.text().await.ok().and_then(|v| i64::from_str(&v).ok()),
            "title" => title = field.text().await.ok(),
            "description" => description = field.text().await.ok(),
            "video_source_primary" => video_source_primary = field.text().await.ok(),
            "video_source_backup" => video_source_backup = field.text().await.ok(),
            "duration" => duration = field.text().await.ok().and_then(|v| i64::from_str(&v).ok()),
            "download_link" => download_link = field.text().await.ok(),
            "size_1080p" => size_1080p = field.text().await.ok(),
            "download_bluray" => download_bluray = field.text().await.ok(),
            "size_bluray" => size_bluray = field.text().await.ok(),
            "download_720p" => download_720p = field.text().await.ok(),
            "size_720p" => size_720p = field.text().await.ok(),
            "download_480p" => download_480p = field.text().await.ok(),
            "size_480p" => size_480p = field.text().await.ok(),
            "thumbnail" => {
                let data = field.bytes().await.unwrap_or_default();
                if !data.is_empty() {
                    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                    let fname = format!("ep_thumb_{}.jpg", ts);
                    let _ = std::fs::write(format!("public/uploads/thumbnails/{}", fname), &data);
                    thumbnail_name = Some(fname);
                }
            }
            _ => {}
        }
    }

    let tid = sqlx::query("INSERT INTO episodes (season_id, episode_number, title, description, video_source_primary, video_source_backup, duration, download_link, size_1080p, download_bluray, size_bluray, download_720p, size_720p, download_480p, size_480p, thumbnail, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))")
        .bind(season_id).bind(episode_number).bind(&title).bind(&description)
        .bind(&video_source_primary).bind(&video_source_backup).bind(duration)
        .bind(&download_link).bind(&size_1080p).bind(&download_bluray).bind(&size_bluray)
        .bind(&download_720p).bind(&size_720p).bind(&download_480p).bind(&size_480p).bind(&thumbnail_name)
        .execute(&state.db).await.unwrap_or_default();
    Redirect::to("/admin/episodes")
}

pub async fn edit(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Html<String> {
    let ep = sqlx::query_as::<_, Episode>("SELECT * FROM episodes WHERE id = ?")
        .bind(id).fetch_optional(&state.db).await.unwrap_or(None);
    match ep {
        Some(e) => Html(format!("<h1>Edit Episode</h1><form method='POST'>
            <input name='title' value='{}'><input name='episode_number' value='{}'>
            <input name='video_source_primary' value='{}'>
            <button>Update</button></form>",
            e.title, e.episode_number, e.video_source_primary.as_deref().unwrap_or(""))),
        None => Html("<h1>Not Found</h1>".to_string()),
    }
}

pub async fn update(State(state): State<Arc<AppState>>, Path(id): Path<i64>, Form(f): Form<EpisodeForm>) -> Redirect {
    sqlx::query("UPDATE episodes SET title=COALESCE(?,title), episode_number=COALESCE(?,episode_number), video_source_primary=COALESCE(?,video_source_primary), updated_at=datetime('now') WHERE id=?")
        .bind(&f.title).bind(f.episode_number).bind(&f.video_source_primary).bind(id)
        .execute(&state.db).await.unwrap_or_default();
    Redirect::to("/admin/episodes")
}

pub async fn destroy(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Redirect {
    sqlx::query("DELETE FROM subtitles WHERE episode_id = ?").bind(id).execute(&state.db).await.unwrap_or_default();
    sqlx::query("DELETE FROM episodes WHERE id = ?").bind(id).execute(&state.db).await.unwrap_or_default();
    Redirect::to("/admin/episodes")
}
