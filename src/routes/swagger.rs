use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::setup::AppState;

use crate::routes::attendee;
use crate::routes::debate;
use crate::routes::motion;
use crate::routes::team;
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
        tournament::get_tournaments,
        motion::get_motions,
        motion::create_motion,
        motion::get_motion_by_id,
        motion::patch_motion_by_id,
        motion::delete_motion_by_id,
        team::get_teams,
        team::create_team,
        team::get_team_by_id,
        team::patch_team_by_id,
        team::delete_team_by_id,
        debate::get_debates,
        debate::create_debate,
        debate::get_debate_by_id,
        debate::patch_debate_by_id,
        debate::delete_debate_by_id,
        attendee::get_attendees,
        attendee::create_attendee,
        attendee::get_attendee_by_id,
        attendee::patch_attendee_by_id,
        attendee::delete_attendee_by_id
    ),
    components(schemas(
        version::VersionDetails,
        version::VersionBits,
        version::GitInfo,
        tournament::Tournament,
        tournament::TournamentPatch,
        motion::Motion,
        motion::MotionPatch,
        team::Team,
        team::TeamPatch,
        debate::Debate,
        debate::DebatePatch,
        attendee::Attendee,
    ))
)]

pub struct ApiDoc;
