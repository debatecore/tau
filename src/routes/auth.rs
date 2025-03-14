use crate::{
    omni_error::OmniError,
    setup::AppState,
    users::{
        auth::{
            cookie::{clear_session_token_cookie, set_session_token_cookie},
            error::AuthError::{
                BadHeaderAuthSchemeData, ClearSessionBearerOnly, NonAsciiHeaderCharacters,
            },
            session::Session,
            AUTH_SESSION_COOKIE_NAME,
        },
        User,
    },
};
use axum::{
    extract::{Path, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use tower_cookies::Cookies;
use utoipa::ToSchema;
use uuid::Uuid;

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(auth_login))
        .route("/auth/clear", get(auth_clear))
        .route("/auth/login/:token", post(login_with_link))
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    login: String,
    password: String,
}

/// Log in to tau
///
/// Returns an auth token and sets a session cookie.
/// Providing the token either by including it in the
/// request header or sending the cookie is required
/// to perform any further operations.
/// By default, the only existing account is the infrastructure admin
/// with username and password "admin".
#[utoipa::path(post, path = "/auth/login", request_body=LoginRequest,
    responses
        (
            (
                status=200,
                description = "Auth token",
                body=String,
                example=json!("UaKN-h7_eD5LlKt8ba4P376G0LGvW3JmccCDMUaPaQk")
            ),
            (status=400, description = "Bad request"),
            (status=401, description = "Invalid credentials"),
            (status=500, description = "Internal server error"),
        )
    )
]
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

#[utoipa::path(
    post,
    path = "/auth/login/{token}",
    responses(
        (
            status = 200,
            description = "Returns an auth token to be used for authentication in subsequent requests",
            body=String,
            example=json!("UaKN-h7_eD5LlKt8ba4P376G0LGvW3JmccCDMUaPaQk")
        ),
        (status = 401, description = "Provided token was invalid"),
        (status = 403, description = "Provided token was used used or expired"),
        (status = 500, description = "Internal server error")
    )
)]
/// Log in with a single-use link
///
/// This endpoint can be used to utilize single-use login links
/// generated with /user/{user_id}/login_link.
async fn login_with_link(
    cookies: Cookies,
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Response, OmniError> {
    let user = User::auth_via_link(&token, &state.connection_pool).await?;
    let (_, token) = match Session::create(&user.id, &state.connection_pool).await {
        Ok(o) => o,
        Err(e) => Err(e)?,
    };

    set_session_token_cookie(&token, cookies);
    Ok((StatusCode::OK, token).into_response())
}

const TOO_MANY_TOKENS: &str = "Please provide one session token to destroy at a time.";
const NO_TOKENS: &str = "Please provide a session token to destroy.";
const SESSION_DESTROYED: &str = "Logged out - Session destroyed";

/// Log out of tau
///
/// Can be used to invalidate auth tokens.
/// Can only invalidate one token at a time.
#[utoipa::path(get, path = "/auth/clear",
    responses
        (
            (
                status=200,
                description = SESSION_DESTROYED,
            ),
            (status=400, description = "Bad request"),
            (status=500, description = "Internal server error"),
        )
    )
]
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
        (Some(_), Some(_)) => (StatusCode::BAD_REQUEST, TOO_MANY_TOKENS).into_response(),
        (None, Some(c)) => {
            auth_clear_to_response(&c, cookies, &state.connection_pool).await
        }
        (Some(h), None) => {
            let (scheme, data) = match h.split_once(' ') {
                Some((a, b)) => (a, b),
                None => return OmniError::from(BadHeaderAuthSchemeData).respond(),
            };
            match scheme {
                "Bearer" => {
                    auth_clear_to_response(&data, cookies, &state.connection_pool).await
                }
                _ => OmniError::from(ClearSessionBearerOnly).respond(),
            }
        }
        (None, None) => (StatusCode::BAD_REQUEST, NO_TOKENS).into_response(),
    }
}

async fn auth_clear_to_response(
    token: &str,
    cookies: Cookies,
    pool: &Pool<Postgres>,
) -> Response {
    clear_session_token_cookie(cookies);
    match Session::get_by_token(token, pool).await {
        Ok(session) => match session.destroy(pool).await {
            Ok(_) => (StatusCode::OK, SESSION_DESTROYED).into_response(),
            Err(e) => e.respond(),
        },
        Err(e) => e.respond(),
    }
}
