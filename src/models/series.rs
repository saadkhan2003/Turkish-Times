use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct Series {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub year: Option<i64>,
    pub thumbnail: Option<String>,
    pub backdrop: Option<String>,
    pub trailer_url: Option<String>,
    pub genres: Option<String>,
    pub status: Option<String>,
    pub views: Option<i64>,
    pub featured: Option<bool>,
    pub rating: Option<f64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Series {
    pub fn genre_list(&self) -> Vec<String> {
        self.genres
            .as_deref()
            .and_then(|g| serde_json::from_str(g).ok())
            .unwrap_or_default()
    }

    pub fn thumbnail_url(&self, app_url: &str) -> String {
        self.thumbnail
            .as_ref()
            .map(|t| format!("{}/uploads/thumbnails/{}", app_url, t))
            .unwrap_or_default()
    }

    pub fn backdrop_url(&self, app_url: &str) -> String {
        self.backdrop
            .as_ref()
            .map(|b| format!("{}/uploads/backdrops/{}", app_url, b))
            .unwrap_or_default()
    }
}
