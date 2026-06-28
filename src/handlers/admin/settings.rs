use axum::{extract::{Multipart, State}, response::{Html, Redirect}};
use std::sync::Arc;
use crate::AppState;

pub async fn edit(State(state): State<Arc<AppState>>) -> Html<String> {
    let setting = sqlx::query_as::<_, crate::models::SiteSetting>(
        "SELECT * FROM site_settings WHERE id = 1"
    ).fetch_optional(&state.db).await.unwrap_or(None);

    let logo_url = setting.as_ref().and_then(|s| s.logo_path.as_ref())
        .map(|p| format!("{}/uploads/settings/{}", state.config.app_url, p))
        .unwrap_or_else(|| format!("{}/images/logo.png", state.config.app_url));
    let favicon_url = setting.as_ref().and_then(|s| s.favicon_path.as_ref())
        .map(|p| format!("{}/uploads/settings/{}", state.config.app_url, p))
        .unwrap_or_else(|| format!("{}/images/favicon.png", state.config.app_url));

    let ctx = tera::Context::from_serialize(serde_json::json!({
        "setting": setting, "logo_url": logo_url, "favicon_url": favicon_url,
        "app_url": &state.config.app_url,
    })).unwrap();
    let body = state.tera.render("admin/settings/edit.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {}", e));
    Html(body)
}

pub async fn update(State(state): State<Arc<AppState>>, mut multipart: Multipart) -> Redirect {
    let mut site_name = None;
    let mut description = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "site_name" => site_name = Some(field.text().await.unwrap_or_default()),
            "description" => description = Some(field.text().await.unwrap_or_default()),
            "logo" | "favicon" => {
                let data = field.bytes().await.unwrap_or_default();
                if !data.is_empty() {
                    let file_name = format!("{}_{}.png", name, std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
                    let save_path = format!("public/uploads/settings/{}", file_name);
                    let _ = std::fs::write(&save_path, &data);
                    let col = if name == "logo" { "logo_path" } else { "favicon_path" };
                    let sql = format!("UPDATE site_settings SET {} = ?, updated_at = datetime('now') WHERE id = 1", col);
                    let _ = sqlx::query(&sql).bind(&file_name).execute(&state.db).await;
                }
            }
            _ => {}
        }
    }

    if let Some(name) = site_name {
        let _ = sqlx::query("UPDATE site_settings SET site_name = ?, updated_at = datetime('now') WHERE id = 1")
            .bind(&name).execute(&state.db).await;
    }
    if let Some(desc) = description {
        let _ = sqlx::query("UPDATE site_settings SET description = ?, updated_at = datetime('now') WHERE id = 1")
            .bind(&desc).execute(&state.db).await;
    }

    Redirect::to("/admin/settings")
}
