use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct Episode {
    pub id: i64,
    pub season_id: i64,
    pub episode_number: i64,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub video_source_primary: Option<String>,
    pub video_source_backup: Option<String>,
    pub download_link: Option<String>,
    pub size_1080p: Option<String>,
    pub download_bluray: Option<String>,
    pub size_bluray: Option<String>,
    pub download_720p: Option<String>,
    pub size_720p: Option<String>,
    pub download_480p: Option<String>,
    pub size_480p: Option<String>,
    pub duration: Option<i64>,
    pub views: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
