use chrono::Duration;
use tower_cookies::cookie::time::Duration as CookieDuration;

pub mod cookie;
pub mod error;
pub mod session;
pub mod userimpl;

pub const AUTH_SESSION_COOKIE_NAME: &str = "tausession";
pub const AUTH_SESSION_LENGTH: Duration = Duration::weeks(1);
pub const AUTH_SESSION_COOKIE_LENGTH: CookieDuration = CookieDuration::weeks(1);
