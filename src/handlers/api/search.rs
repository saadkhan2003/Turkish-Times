use axum::{extract::{Query, State}, response::Json};
use serde::Deserialize;
use std::sync::Arc;
use serde_json::{json, Value};
use crate::AppState;

#[derive(Deserialize)]
pub struct SearchParams {
    q: Option<String>,
}

pub async fn index(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Json<Value> {
    let query = params.q.unwrap_or_default();
    if query.is_empty() {
        return Json(json!({"message": "Search query required"}));
    }
    let like = format!("%{}%", query);
    let results = sqlx::query_as::<_, crate::models::Series>(
        "SELECT * FROM series WHERE title LIKE ? OR description LIKE ? ORDER BY views DESC LIMIT 20",
    )
    .bind(&like).bind(&like)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    Json(json!(results))
}
