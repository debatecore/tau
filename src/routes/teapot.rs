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

#[cfg(test)]
mod tests {
    use std::future::IntoFuture;

    use reqwest::{Client, StatusCode};

    use crate::{setup::get_socket_addr, test};

    #[tokio::test]
    async fn test_teapot() {
        // GIVEN
        let socket_address = get_socket_addr().to_string();
        let app = test::create_app().await;
        let listener = test::create_listener().await;
        let server = axum::serve(listener, app).into_future();
        tokio::spawn(server);

        // WHEN
        let client = Client::new();
        let res = client
            .get(format!("http://{}/brew-coffee", socket_address))
            .send()
            .await
            .unwrap();

        // THEN
        assert_eq!(res.status(), StatusCode::IM_A_TEAPOT);
    }
}
