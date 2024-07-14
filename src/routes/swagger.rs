use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::health_check;
use super::teapot;

pub fn route() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", ApiDoc::openapi()))
}

#[derive(OpenApi)]
#[openapi(paths(health_check::live, health_check::health, teapot::route))]
pub struct ApiDoc;
