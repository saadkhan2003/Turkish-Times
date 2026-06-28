use axum::{
    extract::{Path, Query, State},
    response::Html,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::AppState;
use crate::models::{Series, Season, Episode};

#[derive(Deserialize, Default)]
pub struct SeriesQuery {
    pub status: Option<String>,
    pub genre: Option<String>,
    pub sort: Option<String>,
}

pub async fn index(State(state): State<Arc<AppState>>, Query(q): Query<SeriesQuery>) -> Html<String> {
    let status = q.status.as_deref().unwrap_or("").to_string();
    let genre = q.genre.as_deref().unwrap_or("").to_string();

    let mut sql = String::from("SELECT * FROM series WHERE 1=1");
    if !status.is_empty() {
        sql.push_str(" AND status = ?");
    }
    sql.push_str(" ORDER BY created_at DESC");

    let series = if !status.is_empty() {
        sqlx::query_as::<_, Series>(&sql).bind(&status)
            .fetch_all(&state.db).await.unwrap_or_default()
    } else {
        sqlx::query_as::<_, Series>(&sql)
            .fetch_all(&state.db).await.unwrap_or_default()
    };

    let ctx = tera::Context::from_serialize(serde_json::json!({
        "series": series,
        "status": status,
        "genre": genre,
        "app_url": &state.config.app_url,
    })).unwrap();

    let body = state.tera.render("frontend/series/index.html", &ctx)
        .unwrap_or_else(|e| { eprintln!("SERIES TEMPLATE ERR: {:?}", e); format!("Template error: {:?}", e) });
    Html(body)
}

pub async fn show(State(state): State<Arc<AppState>>, Path(slug): Path<String>) -> Html<String> {
    let series = sqlx::query_as::<_, Series>("SELECT * FROM series WHERE slug = ?")
        .bind(&slug).fetch_optional(&state.db).await.unwrap_or(None);

    match series {
        Some(s) => {
            sqlx::query("UPDATE series SET views = COALESCE(views, 0) + 1 WHERE id = ?")
                .bind(s.id).execute(&state.db).await.unwrap_or_default();

            let seasons = sqlx::query_as::<_, Season>(
                "SELECT * FROM seasons WHERE series_id = ? ORDER BY season_number"
            ).bind(s.id).fetch_all(&state.db).await.unwrap_or_default();

            let episodes = sqlx::query_as::<_, crate::models::Episode>(
                "SELECT e.* FROM episodes e JOIN seasons s ON e.season_id = s.id WHERE s.series_id = ? ORDER BY s.season_number, e.episode_number"
            ).bind(s.id).fetch_all(&state.db).await.unwrap_or_default();

            let first_ep_id = episodes.first().map(|e| e.id);

            let related = sqlx::query_as::<_, Series>(
                "SELECT * FROM series WHERE id != ? AND (genres LIKE ? OR genres LIKE ?) ORDER BY views DESC LIMIT 6"
            ).bind(s.id).bind(format!("%{}%", s.genres.as_deref().unwrap_or("").chars().take(20).collect::<String>()))
             .bind(format!("%{}%", "Historical"))
             .fetch_all(&state.db).await.unwrap_or_default();

            let ctx = tera::Context::from_serialize(serde_json::json!({
                "series": s, "seasons": seasons, "episodes": episodes,
                "related_series": related,
                "episodes_first_id": first_ep_id, "app_url": &state.config.app_url,
            })).unwrap();

            let body = state.tera.render("frontend/series/show.html", &ctx)
                .unwrap_or_else(|e| { eprintln!("SERIES TEMPLATE ERR: {:?}", e); format!("Template error: {:?}", e) });
            Html(body)
        }
        None => Html("<h1>Not Found</h1>".to_string()),
    }
}

pub async fn season(
    State(state): State<Arc<AppState>>,
    Path((slug, season_num)): Path<(String, i64)>,
) -> Html<String> {
    let series = sqlx::query_as::<_, Series>("SELECT * FROM series WHERE slug = ?")
        .bind(&slug).fetch_optional(&state.db).await.unwrap_or(None);

    match series {
        Some(s) => {
            let season = sqlx::query_as::<_, Season>(
                "SELECT * FROM seasons WHERE series_id = ? AND season_number = ?"
            ).bind(s.id).bind(season_num).fetch_optional(&state.db).await.unwrap_or(None);

            match season {
                Some(sea) => {
                    let episodes = sqlx::query_as::<_, Episode>(
                        "SELECT * FROM episodes WHERE season_id = ? ORDER BY episode_number"
                    ).bind(sea.id).fetch_all(&state.db).await.unwrap_or_default();

                    let ctx = tera::Context::from_serialize(serde_json::json!({
                        "series": s, "season": sea, "episodes": episodes, "app_url": &state.config.app_url,
                    })).unwrap();

                    let body = state.tera.render("frontend/series/season.html", &ctx)
                        .unwrap_or_else(|e| { eprintln!("SEASON TEMPLATE ERR: {:?}", e); format!("Template error: {:?}", e) });
                    Html(body)
                }
                None => Html("<h1>Season Not Found</h1>".to_string()),
            }
        }
        None => Html("<h1>Series Not Found</h1>".to_string()),
    }
}

pub async fn by_genre(State(state): State<Arc<AppState>>, Path(genre): Path<String>) -> Html<String> {
    let series = sqlx::query_as::<_, Series>(
        "SELECT * FROM series WHERE genres LIKE ? ORDER BY created_at DESC"
    ).bind(format!("%{}%", &genre)).fetch_all(&state.db).await.unwrap_or_default();

    let ctx = tera::Context::from_serialize(serde_json::json!({
        "series": series, "genre": genre, "status": "", "app_url": &state.config.app_url,
    })).unwrap();

    let body = state.tera.render("frontend/series/index.html", &ctx)
        .unwrap_or_else(|e| { eprintln!("SERIES TEMPLATE ERR: {:?}", e); format!("Template error: {:?}", e) });
    Html(body)
}
