use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::health;
use super::teapot;

pub fn route() -> Router {
    Router::new().merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", ApiDoc::openapi()))
}

#[derive(OpenApi)]
#[openapi(paths(health::route, teapot::route))]
pub struct ApiDoc;
