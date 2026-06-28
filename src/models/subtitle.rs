use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct Subtitle {
    pub id: i64,
    pub episode_id: i64,
    pub language: Option<String>,
    pub label: Option<String>,
    pub file_path: Option<String>,
    pub is_default: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
