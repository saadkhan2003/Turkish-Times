use axum::{
    extract::State,
    response::{Html, Redirect},
    Form,
};
use axum::http::{header::SET_COOKIE, HeaderMap, HeaderValue};
use serde::Deserialize;
use std::sync::Arc;
use crate::AppState;

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn show_login() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html><html><head><title>Admin Login</title>
    <script src="https://cdn.tailwindcss.com"></script></head>
    <body class="bg-gray-100 min-h-screen flex items-center justify-center">
    <div class="bg-white p-8 rounded-xl shadow-lg w-full max-w-md">
    <h1 class="text-2xl font-bold text-center mb-6">Admin Login</h1>
    <form method="POST" class="space-y-4">
    <div><label class="block text-sm font-medium mb-1">Username</label>
    <input type="text" name="username" class="w-full border rounded-lg px-4 py-2" required></div>
    <div><label class="block text-sm font-medium mb-1">Password</label>
    <input type="password" name="password" class="w-full border rounded-lg px-4 py-2" required></div>
    <button type="submit" class="w-full bg-red-600 text-white py-2 rounded-lg font-bold hover:bg-red-700">Login</button>
    </form></div></body></html>"#)
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Form(form): Form<LoginForm>,
) -> (HeaderMap, Redirect) {
    let mut headers = HeaderMap::new();
    let admin = sqlx::query_as::<_, crate::models::Admin>(
        "SELECT * FROM admins WHERE username = ?",
    ).bind(&form.username).fetch_optional(&state.db).await.unwrap_or(None);

    match admin {
        Some(a) if bcrypt::verify(&form.password, &a.password).unwrap_or(false) => {
            headers.insert(SET_COOKIE,
                HeaderValue::from_str(&format!("admin_id={}; Path=/; HttpOnly; SameSite=Lax", a.id)).unwrap());
            (headers, Redirect::to("/admin/dashboard"))
        }
        _ => {
            headers.insert(SET_COOKIE, HeaderValue::from_static("admin_id=; Path=/; Max-Age=0"));
            (headers, Redirect::to("/admin/login"))
        }
    }
}

pub async fn logout() -> (HeaderMap, Redirect) {
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, HeaderValue::from_static("admin_id=; Path=/; Max-Age=0"));
    (headers, Redirect::to("/admin/login"))
}
