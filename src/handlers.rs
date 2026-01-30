use crate::models::{AppState, LoginPayload};
use axum::{
    Form, Json,
    body::Body,
    extract::{Path, Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use std::sync::Arc;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

pub async fn metadata(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(state.metadata.clone())
}

pub async fn file(Path(uuid): Path<Uuid>, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let file_info = state
        .metadata
        .iter()
        .find(|f| f.uuid == uuid)
        .ok_or((StatusCode::NOT_FOUND, "File not found"))?;

    let file = match tokio::fs::File::open(&file_info.file_path).await {
        Ok(file) => file,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "File system error")),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let content_disposition = format!("attachment; filename=\"{}\"", file_info.file_name);

    let response = Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .header(header::CONTENT_LENGTH, file_info.file_size)
        .body(body)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to build response",
            )
        })?;

    Ok(response)
}

pub async fn auth(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_cookie = jar.get("session").map(|cookie| cookie.value().to_string());

    if let Some(token) = auth_cookie {
        if token == state.auth_token {
            return Ok(next.run(request).await);
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

pub async fn index() -> Html<&'static str> {
    Html(include_str!("../index.html"))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(payload): Form<LoginPayload>,
) -> impl IntoResponse {
    if payload.password == state.password {
        let cookie = Cookie::build(("session", state.auth_token.clone()))
            .path("/")
            .http_only(true)
            .secure(true)
            .build();

        (jar.add(cookie), Redirect::to("/"))
    } else {
        (jar, Redirect::to("/?error=invalid_password"))
    }
}
