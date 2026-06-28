use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Admin {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
