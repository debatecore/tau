use axum::{http::StatusCode, routing::get, Router};

static IM_A_TEAPOT_RESPONSE: &str = "I'm a teapot";

#[utoipa::path(get, path = "/brew-coffee", responses((status = 418, description = IM_A_TEAPOT_RESPONSE)))]
pub fn route() -> Router {
    Router::new().route("/brew-coffee", get(im_a_teapot()))
}

fn im_a_teapot() -> (StatusCode, String) {
    (StatusCode::IM_A_TEAPOT, IM_A_TEAPOT_RESPONSE.to_owned())
}
