use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct Season {
    pub id: i64,
    pub series_id: i64,
    pub season_number: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub trailer_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
