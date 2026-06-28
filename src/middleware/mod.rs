use axum::{
    extract::Request,
    middleware::Next,
    response::{Redirect, Response},
};
use std::sync::Arc;
use crate::AppState;

pub async fn require_admin(
    mut request: Request,
    next: Next,
) -> Result<Response, Redirect> {
    let cookies = request
        .headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let has_session = cookies
        .split(';')
        .any(|c| c.trim().starts_with("admin_id="));

    if has_session {
        Ok(next.run(request).await)
    } else {
        Err(Redirect::to("/admin/login"))
    }
}
