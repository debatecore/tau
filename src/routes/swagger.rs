use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::tournament;
use crate::setup::AppState;

use super::health_check;
use super::teapot;
use super::version;

pub fn route() -> Router<AppState> {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", ApiDoc::openapi()))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check::live,
        health_check::health,
        teapot::route,
        version::version,
        version::version_details,
        tournament::create_tournament,
        tournament::get_tournament_by_id,
        tournament::patch_tournament_by_id,
        tournament::delete_tournament_by_id,
        tournament::get_tournaments
    ),
    components(schemas(
        version::VersionDetails,
        version::VersionBits,
        tournament::Tournament,
        tournament::TournamentPatch
    ))
)]
pub struct ApiDoc;
