use axum::{
    extract::{Multipart, Path, State},
    response::{Html, Redirect},
    Form,
};
use std::sync::Arc;
use serde::Deserialize;
use crate::AppState;
use crate::models::Series;

pub async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let series = sqlx::query_as::<_, Series>("SELECT * FROM series ORDER BY created_at DESC")
        .fetch_all(&state.db).await.unwrap_or_default();
    let ctx = tera::Context::from_serialize(serde_json::json!({
        "series": series, "app_url": &state.config.app_url,
    })).unwrap();
    let body = state.tera.render("admin/series/index.html", &ctx)
        .unwrap_or_else(|e| format!("Error: {}", e));
    Html(body)
}

pub async fn create(State(state): State<Arc<AppState>>) -> Html<String> {
    let ctx = tera::Context::from_serialize(serde_json::json!({
        "app_url": &state.config.app_url,
    })).unwrap();
    let body = state.tera.render("admin/series/create.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {}", e));
    Html(body)
}

#[derive(Deserialize)]
pub struct SeriesForm {
    pub title: Option<String>,
    pub description: Option<String>,
    pub year: Option<i64>,
    pub status: Option<String>,
    pub featured: Option<String>,
    pub rating: Option<f64>,
}

pub async fn store(State(state): State<Arc<AppState>>, Form(f): Form<SeriesForm>) -> Redirect {
    let title = f.title.unwrap_or_default();
    let slug = title.to_lowercase().replace(' ', "-").replace(':', "").replace("'", "");
    sqlx::query("INSERT INTO series (title, slug, description, year, status, featured, rating, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))")
        .bind(&title).bind(&slug).bind(&f.description).bind(f.year)
        .bind(f.status.as_deref().unwrap_or("ongoing"))
        .bind(f.featured.is_some())
        .bind(f.rating)
        .execute(&state.db).await.unwrap_or_default();
    Redirect::to("/admin/series")
}

pub async fn upload_image(State(state): State<Arc<AppState>>, Path(id): Path<i64>, mut multipart: Multipart) -> Redirect {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        let file_name = field.file_name().unwrap_or("").to_string();
        let data = field.bytes().await.unwrap_or_default();

        if data.is_empty() || file_name.is_empty() { continue; }

        let ext = file_name.rsplit('.').next().unwrap_or("jpg");
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let saved_name = format!("{}_{}.{}", id, name, ext);
        let save_path = format!("public/uploads/{}/{}", match name.as_str() {
            "backdrop" => "backdrops",
            _ => "thumbnails",
        }, saved_name);

        if let Err(e) = std::fs::write(&save_path, &data) {
            eprintln!("Failed to save upload: {}", e);
            continue;
        }

        let col = if name == "backdrop" { "backdrop" } else { "thumbnail" };
        let sql = format!("UPDATE series SET {} = ?, updated_at = datetime('now') WHERE id = ?", col);
        sqlx::query(&sql).bind(&saved_name).bind(id)
            .execute(&state.db).await.unwrap_or_default();
    }
    Redirect::to(&format!("/admin/series/{}/edit", id))
}

pub async fn edit(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Html<String> {
    let series = sqlx::query_as::<_, Series>("SELECT * FROM series WHERE id = ?")
        .bind(id).fetch_optional(&state.db).await.unwrap_or(None);
    match series {
        Some(s) => {
            let thumb = s.thumbnail.as_deref().unwrap_or("");
            let back = s.backdrop.as_deref().unwrap_or("");
            let body = format!(r#"<!DOCTYPE html><html><head><title>Edit Series</title>
            <script src="https://cdn.tailwindcss.com"></script>
            <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.2/css/all.min.css">
            </head><body class="bg-gray-50 min-h-screen"><div class="max-w-2xl mx-auto p-8">
            <a href="/admin/series" class="text-gray-500 hover:text-gray-700 text-sm mb-4 block"><i class="fas fa-arrow-left mr-1"></i>Back to Series</a>
            <h1 class="text-2xl font-bold mb-6">Edit: {title}</h1>

            <form method="POST" class="bg-white p-6 rounded-xl shadow-sm space-y-4 mb-8">
            <div><label class="block text-sm font-medium text-gray-700 mb-1">Title</label>
            <input type="text" name="title" value="{title}" class="w-full border rounded-lg px-4 py-2" required></div>
            <div><label class="block text-sm font-medium text-gray-700 mb-1">Description</label>
            <textarea name="description" rows="4" class="w-full border rounded-lg px-4 py-2">{desc}</textarea></div>
            <div class="grid grid-cols-3 gap-4">
            <div><label class="block text-sm font-medium text-gray-700 mb-1">Year</label>
            <input type="number" name="year" value="{year}" class="w-full border rounded-lg px-4 py-2"></div>
            <div><label class="block text-sm font-medium text-gray-700 mb-1">Status</label>
            <select name="status" class="w-full border rounded-lg px-4 py-2">
            <option value="ongoing" {sel_o}>Ongoing</option>
            <option value="completed" {sel_c}>Completed</option></select></div>
            <div><label class="block text-sm font-medium text-gray-700 mb-1">Rating</label>
            <input type="number" name="rating" step="0.1" min="0" max="10" value="{rating}" class="w-full border rounded-lg px-4 py-2"></div>
            </div>
            <button type="submit" class="bg-blue-600 text-white px-6 py-2 rounded-lg hover:bg-blue-700 transition font-medium">Update Series</button>
            </form>

            <div class="bg-white p-6 rounded-xl shadow-sm space-y-4">
                <h3 class="text-lg font-semibold">Images</h3>
                <form method="POST" action="/admin/series/{id}/upload" enctype="multipart/form-data" class="space-y-4">
                <div><label class="block text-sm font-medium text-gray-700 mb-1">Thumbnail (poster)</label>
                <input type="file" name="thumbnail" accept="image/*" class="w-full border rounded-lg px-4 py-2">
                {thumb_html}</div>
                <div><label class="block text-sm font-medium text-gray-700 mb-1">Backdrop (hero)</label>
                <input type="file" name="backdrop" accept="image/*" class="w-full border rounded-lg px-4 py-2">
                {back_html}</div>
                <button type="submit" class="bg-green-600 text-white px-6 py-2 rounded-lg hover:bg-green-700 transition font-medium">Upload Images</button>
                </form>
            </div>
            </div></body></html>"#,
                title=s.title, id=s.id,
                desc=s.description.as_deref().unwrap_or(""),
                year=s.year.unwrap_or(0),
                rating=s.rating.unwrap_or(8.5),
                sel_o=if s.status.as_deref()==Some("ongoing"){"selected"}else{""},
                sel_c=if s.status.as_deref()==Some("completed"){"selected"}else{""},
                thumb_html=if !thumb.is_empty() {format!("<img src='{0}/uploads/thumbnails/{1}' class='h-20 mt-2 rounded'>", state.config.app_url, thumb)}else{"".to_string()},
                back_html=if !back.is_empty() {format!("<img src='{0}/uploads/backdrops/{1}' class='h-20 mt-2 rounded'>", state.config.app_url, back)}else{"".to_string()},
            );
            Html(body)
        }
        None => Html("<h1>Not Found</h1>".to_string()),
    }
}

pub async fn update(State(state): State<Arc<AppState>>, Path(id): Path<i64>, Form(f): Form<SeriesForm>) -> Redirect {
    sqlx::query("UPDATE series SET title=COALESCE(?,title), description=COALESCE(?,description), year=COALESCE(?,year), status=COALESCE(?,status), rating=COALESCE(?,rating), updated_at=datetime('now') WHERE id=?")
        .bind(&f.title).bind(&f.description).bind(f.year).bind(f.status.as_deref().unwrap_or("ongoing")).bind(f.rating).bind(id)
        .execute(&state.db).await.unwrap_or_default();
    Redirect::to("/admin/series")
}

pub async fn destroy(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Redirect {
    sqlx::query("DELETE FROM episodes WHERE season_id IN (SELECT id FROM seasons WHERE series_id = ?)").bind(id).execute(&state.db).await.unwrap_or_default();
    sqlx::query("DELETE FROM seasons WHERE series_id = ?").bind(id).execute(&state.db).await.unwrap_or_default();
    sqlx::query("DELETE FROM series WHERE id = ?").bind(id).execute(&state.db).await.unwrap_or_default();
    Redirect::to("/admin/series")
}
