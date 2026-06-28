use std::env;

pub struct Config {
    pub database_url: String,
    pub app_url: String,
    pub session_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:data/database.sqlite".to_string()),
            app_url: env::var("APP_URL")
                .unwrap_or_else(|_| "http://localhost:8000".to_string()),
            session_secret: env::var("SESSION_SECRET")
                .unwrap_or_else(|_| "change-this-secret-key-in-production".to_string()),
        }
    }
}
