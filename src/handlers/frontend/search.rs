use axum::{extract::{Query, State}, response::Html};
use serde::Deserialize;
use std::sync::Arc;
use crate::AppState;
use crate::models::Series;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

pub async fn index(State(state): State<Arc<AppState>>, Query(q): Query<SearchQuery>) -> Html<String> {
    let query = q.q.unwrap_or_default();
    let series: Vec<Series> = if query.len() >= 2 {
        let like = format!("%{}%", query);
        sqlx::query_as::<_, Series>(
            "SELECT * FROM series WHERE title LIKE ? OR description LIKE ? ORDER BY views DESC"
        ).bind(&like).bind(&like).fetch_all(&state.db).await.unwrap_or_default()
    } else {
        vec![]
    };

    let ctx = tera::Context::from_serialize(serde_json::json!({
        "query": query, "series": series, "app_url": &state.config.app_url,
    })).unwrap();

    let body = state.tera.render("frontend/search.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {}", e));
    Html(body)
}
