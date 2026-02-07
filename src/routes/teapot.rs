use axum::{http::StatusCode, routing::get, Router};

use crate::setup::AppState;

static IM_A_TEAPOT_RESPONSE: &str = "I'm a teapot!";

/// The HTCPCP server is a teapot; the resulting entity body is short and stout.
#[utoipa::path(get, path = "/brew-coffee", responses((status = 418, description = IM_A_TEAPOT_RESPONSE)))]
pub fn route() -> Router<AppState> {
    Router::new().route("/brew-coffee", get(im_a_teapot()))
}

fn im_a_teapot() -> (StatusCode, &'static str) {
    (StatusCode::IM_A_TEAPOT, IM_A_TEAPOT_RESPONSE)
}
