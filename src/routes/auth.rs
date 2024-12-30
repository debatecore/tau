use crate::{
    omni_error::OmniError,
    setup::AppState,
    users::{
        auth::{
            cookie::set_session_token_cookie, error::AuthError::NonAsciiHeaderCharacters,
            session::Session, AUTH_SESSION_COOKIE_NAME,
        },
        User,
    },
};
use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use tower_cookies::Cookies;

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(auth_login))
        .route("/auth/clear", get(auth_clear))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoginRequest {
    login: String,
    password: String,
}

async fn auth_login(
    cookies: Cookies,
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Response {
    let user = match User::auth_via_credentials(
        &body.login,
        &body.password,
        &state.connection_pool,
    )
    .await
    {
        Ok(user) => user,
        Err(e) => return e.respond(),
    };

    let (_, token) = match Session::create(&user.id, &state.connection_pool).await {
        Ok(o) => o,
        Err(e) => return e.respond(),
    };

    set_session_token_cookie(&token, cookies);
    (StatusCode::OK, token).into_response()
}

const TOO_MANY_TOKENS: &str = "Please provide one session token to destroy at a time.";
const NO_TOKENS: &str = "Please provide a session token to destroy.";
const SESSION_DESTROYED: &str = "Logged out - Session destroyed";

async fn auth_clear(
    headers: HeaderMap,
    cookies: Cookies,
    State(state): State<AppState>,
) -> Response {
    let header = match headers.get(AUTHORIZATION) {
        Some(h) => match h.to_str() {
            Ok(t) => Some(t.to_string()),
            Err(_) => return OmniError::from(NonAsciiHeaderCharacters).respond(),
        },
        None => None,
    };
    let cookie = match cookies.get(AUTH_SESSION_COOKIE_NAME) {
        Some(c) => {
            let c = c.value().to_string();
            match &header {
                Some(h) => match &c == h {
                    true => None,
                    false => Some(c),
                },
                None => Some(c),
            }
        }
        None => None,
    };

    match (header, cookie) {
        (Some(h), Some(c)) => (StatusCode::BAD_REQUEST, TOO_MANY_TOKENS).into_response(),
        (None, Some(c)) => auth_clear_to_response(&c, &state.connection_pool).await,
        (Some(h), None) => auth_clear_to_response(&h, &state.connection_pool).await,
        (None, None) => (StatusCode::BAD_REQUEST, NO_TOKENS).into_response(),
    }
}

async fn auth_clear_to_response(token: &str, pool: &Pool<Postgres>) -> Response {
    match Session::get_by_token(token, pool).await {
        Ok(session) => match session.destroy(pool).await {
            Ok(_) => (StatusCode::OK, "Session destroyed.").into_response(),
            Err(e) => e.respond(),
        },
        Err(e) => e.respond(),
    }
}
