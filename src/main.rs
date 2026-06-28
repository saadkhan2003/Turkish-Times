mod config;
mod handlers;
mod middleware;
mod models;

use axum::{
    middleware as axum_mw,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub tera: tera::Tera,
    pub config: config::Config,
}

#[tokio::main]
async fn main() {
    let cfg = config::Config::from_env();

    // Ensure database file exists (SQLite needs O_CREAT, sqlx doesn't add it)
    if let Some(path) = cfg.database_url.strip_prefix("sqlite:") {
        let p = std::path::Path::new(path);
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if !p.exists() {
            let _ = std::fs::File::create(p).map(|f| drop(f));
        }
    }

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await
        .expect("Failed to connect to database");

    let template_dir = std::env::var("TEMPLATE_DIR").unwrap_or_else(|_| "templates".to_string());
    let pattern = format!("{}/**/*.html", template_dir);
    let mut tera = tera::Tera::new(&pattern).expect("Failed to load templates");
    tera.autoescape_on(Vec::new());
    tera.register_filter("round", |v: &tera::Value, _: &std::collections::HashMap<String, tera::Value>| {
        Ok(tera::to_value(v.as_f64().map(|f| f.round()).unwrap_or(0.0)).unwrap())
    });
    tera.register_filter("from_json", |v: &tera::Value, _: &std::collections::HashMap<String, tera::Value>| {
        let s = v.as_str().unwrap_or("[]");
        Ok(serde_json::from_str::<tera::Value>(s).unwrap_or(tera::Value::Array(vec![])))
    });

    // Auto-create admin account if none exists
    let admin_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admins")
        .fetch_one(&db).await.unwrap_or(0);
    if admin_count == 0 {
        let hash = bcrypt::hash("admin123", 12).unwrap_or_default();
        sqlx::query("INSERT INTO admins (username, password, name, email, created_at, updated_at) VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))")
            .bind("admin").bind(&hash).bind("Administrator").bind("admin@turkishtimes.com")
            .execute(&db).await.unwrap_or_default();
        println!("✅ Created default admin: admin / admin123");
    }

    let state = Arc::new(AppState { db, tera, config: cfg });

    let frontend = Router::new()
        .route("/", get(handlers::frontend::home))
        .route("/search", get(handlers::frontend::search::index))
        .route("/series", get(handlers::frontend::series::index))
        .route("/series/{slug}", get(handlers::frontend::series::show))
        .route("/series/{slug}/season/{season_num}", get(handlers::frontend::series::season))
        .route("/genre/{genre}", get(handlers::frontend::series::by_genre))
        .route("/watch/{id}", get(handlers::frontend::episode::show));

    let admin_auth = Router::new()
        .route("/admin/login", get(handlers::admin::show_login).post(handlers::admin::login))
        .route("/admin/logout", post(handlers::admin::logout));

    let admin_crud = Router::new()
        .route("/admin", get(handlers::admin::dashboard))
        .route("/admin/dashboard", get(handlers::admin::dashboard))
        .route("/admin/series", get(handlers::admin::series::index))
        .route("/admin/series/create", get(handlers::admin::series::create).post(handlers::admin::series::store))
        .route("/admin/series/{id}/edit", get(handlers::admin::series::edit).post(handlers::admin::series::update))
        .route("/admin/series/{id}/upload", post(handlers::admin::series::upload_image))
        .route("/admin/series/{id}/delete", post(handlers::admin::series::destroy))
        .route("/admin/seasons", get(handlers::admin::seasons::index))
        .route("/admin/seasons/create", get(handlers::admin::seasons::create).post(handlers::admin::seasons::store))
        .route("/admin/seasons/{id}/edit", get(handlers::admin::seasons::edit).post(handlers::admin::seasons::update))
        .route("/admin/seasons/{id}/delete", post(handlers::admin::seasons::destroy))
        .route("/admin/episodes", get(handlers::admin::episodes::index))
        .route("/admin/episodes/create", get(handlers::admin::episodes::create).post(handlers::admin::episodes::store))
        .route("/admin/episodes/{id}/edit", get(handlers::admin::episodes::edit).post(handlers::admin::episodes::update))
        .route("/admin/episodes/{id}/delete", post(handlers::admin::episodes::destroy))
        .route("/admin/subtitles", get(handlers::admin::subtitles::index))
        .route("/admin/subtitles/create", get(handlers::admin::subtitles::create))
        .route("/admin/settings", get(handlers::admin::settings::edit).post(handlers::admin::settings::update))
        .route_layer(axum_mw::from_fn(middleware::require_admin));

    let api = Router::new()
        .route("/api/series", get(handlers::api::series::index))
        .route("/api/series/{id}", get(handlers::api::series::show))
        .route("/api/series/{id}/seasons", get(handlers::api::series::seasons))
        .route("/api/seasons/{id}/episodes", get(handlers::api::series::episodes))
        .route("/api/episodes/{id}", get(handlers::api::series::episode_detail))
        .route("/api/search", get(handlers::api::search::index));

    let app = frontend
        .merge(admin_auth)
        .merge(admin_crud)
        .merge(api)
        .nest_service("/uploads", ServeDir::new("public/uploads"))
        .nest_service("/images", ServeDir::new("public/images"))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("✅ Turkish Times Rust — http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
