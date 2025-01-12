use tower_cookies::{cookie::SameSite, Cookie, Cookies};

use super::{AUTH_SESSION_COOKIE_LENGTH, AUTH_SESSION_COOKIE_NAME};

pub fn set_session_token_cookie(token: &str, cookies: Cookies) {
    let c = Cookie::build((AUTH_SESSION_COOKIE_NAME, token.to_string()))
        .max_age(AUTH_SESSION_COOKIE_LENGTH)
        .http_only(true)
        .path("/")
        .same_site(SameSite::Strict)
        .secure(true)
        .build();
    cookies.add(c);
}

pub fn clear_session_token_cookie(cookies: Cookies) {
    let c = Cookie::build(AUTH_SESSION_COOKIE_NAME).removal().build();
    cookies.add(c);
}
