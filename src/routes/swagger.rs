use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::health_check;
use super::teapot;
use super::version;

pub fn route() -> Router {
    Router::new().merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", ApiDoc::openapi()))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check::live,
        health_check::health,
        teapot::route,
        version::version,
        version::version_details
    ),
    components(schemas(version::VersionDetails, version::VersionBits))
)]
pub struct ApiDoc;
