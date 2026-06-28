use axum::{extract::{Path, State}, response::Html};
use std::sync::Arc;
use crate::AppState;
use crate::models::{Episode, Season, Series, Subtitle};

pub async fn show(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Html<String> {
    let episode = sqlx::query_as::<_, Episode>("SELECT * FROM episodes WHERE id = ?")
        .bind(id).fetch_optional(&state.db).await.unwrap_or(None);

    match episode {
        Some(ep) => {
            sqlx::query("UPDATE episodes SET views = COALESCE(views, 0) + 1 WHERE id = ?")
                .bind(ep.id).execute(&state.db).await.unwrap_or_default();

            let season = sqlx::query_as::<_, Season>(
                "SELECT * FROM seasons WHERE id = ?"
            ).bind(ep.season_id).fetch_optional(&state.db).await.unwrap_or(None)
                .unwrap_or_default();

            let series = if season.id != 0 {
                sqlx::query_as::<_, Series>(
                    "SELECT * FROM series WHERE id = ?"
                ).bind(season.series_id).fetch_optional(&state.db).await.unwrap_or(None)
                    .unwrap_or_default()
            } else {
                Series::default()
            };

            let all_episodes = sqlx::query_as::<_, Episode>(
                "SELECT * FROM episodes WHERE season_id = ? ORDER BY episode_number"
            ).bind(ep.season_id).fetch_all(&state.db).await.unwrap_or_default();

            let subtitles = sqlx::query_as::<_, Subtitle>(
                "SELECT * FROM subtitles WHERE episode_id = ?"
            ).bind(ep.id).fetch_all(&state.db).await.unwrap_or_default();

            let next_ep: Option<Episode> = sqlx::query_as::<_, Episode>(
                "SELECT * FROM episodes WHERE season_id = ? AND episode_number > ? ORDER BY episode_number LIMIT 1"
            ).bind(ep.season_id).bind(ep.episode_number).fetch_optional(&state.db).await.unwrap_or(None);

            let prev_ep: Option<Episode> = sqlx::query_as::<_, Episode>(
                "SELECT * FROM episodes WHERE season_id = ? AND episode_number < ? ORDER BY episode_number DESC LIMIT 1"
            ).bind(ep.season_id).bind(ep.episode_number).fetch_optional(&state.db).await.unwrap_or(None);

            let ctx = tera::Context::from_serialize(serde_json::json!({
                "episode": ep, "season": season, "series": series,
                "all_episodes": all_episodes, "subtitles": subtitles,
                "next_episode": next_ep, "prev_episode": prev_ep,
                "app_url": &state.config.app_url,
            })).unwrap();

            let body = state.tera.render("frontend/episodes/show.html", &ctx)
                .unwrap_or_else(|e| { eprintln!("EPISODE TEMPLATE ERR: {:?}", e); format!("Template error: {:?}", e) });
            Html(body)
        }
        None => Html("<h1>Episode Not Found</h1>".to_string()),
    }
}
