use axum::{extract::State, response::Html};
use std::sync::Arc;
use crate::AppState;

pub async fn home(State(state): State<Arc<AppState>>) -> Html<String> {
    let featured = sqlx::query_as::<_, crate::models::Series>(
        "SELECT * FROM series WHERE featured = 1 ORDER BY created_at DESC LIMIT 6",
    ).fetch_all(&state.db).await.unwrap_or_default();

    let latest = sqlx::query_as::<_, crate::models::Series>(
        "SELECT * FROM series ORDER BY created_at DESC LIMIT 12",
    ).fetch_all(&state.db).await.unwrap_or_default();

    #[derive(sqlx::FromRow, serde::Serialize)]
    struct EpisodeWithSeries {
        id: i64,
        season_id: i64,
        episode_number: i64,
        title: String,
        description: Option<String>,
        thumbnail: Option<String>,
        video_source_primary: Option<String>,
        video_source_backup: Option<String>,
        download_link: Option<String>,
        size_1080p: Option<String>,
        download_bluray: Option<String>,
        size_bluray: Option<String>,
        download_720p: Option<String>,
        size_720p: Option<String>,
        download_480p: Option<String>,
        size_480p: Option<String>,
        duration: Option<i64>,
        views: Option<i64>,
        created_at: Option<String>,
        updated_at: Option<String>,
        series_title: Option<String>,
        series_slug: Option<String>,
        series_thumbnail: Option<String>,
        series_id: i64,
        season_number: i64,
    }

    let latest_eps = sqlx::query_as::<_, EpisodeWithSeries>(
        "SELECT e.*, ser.title as series_title, ser.slug as series_slug, ser.thumbnail as series_thumbnail, ser.id as series_id, s.season_number FROM episodes e \
         JOIN seasons s ON e.season_id = s.id \
         JOIN series ser ON s.series_id = ser.id \
         ORDER BY e.created_at DESC LIMIT 12",
    ).fetch_all(&state.db).await.unwrap_or_default();

    let completed = sqlx::query_as::<_, crate::models::Series>(
        "SELECT * FROM series WHERE status = 'completed' ORDER BY views DESC LIMIT 8",
    ).fetch_all(&state.db).await.unwrap_or_default();

    let genres_raw: Vec<Option<String>> = sqlx::query_scalar("SELECT genres FROM series WHERE genres IS NOT NULL")
        .fetch_all(&state.db).await.unwrap_or_default();
    let mut genres: Vec<String> = Vec::new();
    for g in genres_raw {
        if let Some(json_str) = g {
            if let Ok(list) = serde_json::from_str::<Vec<String>>(&json_str) {
                for item in list {
                    if !genres.contains(&item) { genres.push(item); }
                }
            }
        }
    }
    genres.truncate(10);

    let ctx = tera::Context::from_serialize(serde_json::json!({
        "featured_series": featured,
        "latest_series": latest,
        "latest_episodes": latest_eps,
        "completed_series": completed,
        "genres": genres,
        "app_url": &state.config.app_url,
        "year": 2026,
    })).unwrap();

    let body = match state.tera.render("frontend/home.html", &ctx) {
        Ok(b) => b,
        Err(e) => format!("Template error: {:?}", e),
    };
    Html(body)
}
