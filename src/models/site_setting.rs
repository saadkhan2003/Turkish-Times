use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SiteSetting {
    pub id: i64,
    pub site_name: Option<String>,
    pub logo_path: Option<String>,
    pub favicon_path: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
