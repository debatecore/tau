use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::auth;
use crate::setup::AppState;

use crate::routes::attendee_routes;
use crate::routes::debate_routes;
use crate::routes::location_routes;
use crate::routes::motion_routes;
use crate::routes::room_routes;
use crate::routes::team_routes;
use crate::routes::tournament_routes;
use crate::tournament;
use crate::tournament::attendee;
use crate::tournament::debate;
use crate::tournament::location;
use crate::tournament::motion;
use crate::tournament::room;
use crate::tournament::team;
use crate::users::permissions;
use crate::users::roles;

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
        tournament_routes::create_tournament,
        tournament_routes::get_tournament_by_id,
        tournament_routes::patch_tournament_by_id,
        tournament_routes::delete_tournament_by_id,
        tournament_routes::get_tournaments,
        motion_routes::get_motions,
        motion_routes::create_motion,
        motion_routes::get_motion_by_id,
        motion_routes::patch_motion_by_id,
        motion_routes::delete_motion_by_id,
        team_routes::get_teams,
        team_routes::create_team,
        team_routes::get_team_by_id,
        team_routes::patch_team_by_id,
        team_routes::delete_team_by_id,
        debate_routes::get_debates,
        debate_routes::create_debate,
        debate_routes::get_debate_by_id,
        debate_routes::patch_debate_by_id,
        debate_routes::delete_debate_by_id,
        attendee_routes::get_attendees,
        attendee_routes::create_attendee,
        attendee_routes::get_attendee_by_id,
        attendee_routes::patch_attendee_by_id,
        attendee_routes::delete_attendee_by_id,
        auth::auth_login,
        auth::auth_me,
        location_routes::create_location,
        location_routes::get_locations,
        location_routes::get_location_by_id,
        location_routes::patch_location_by_id,
        location_routes::delete_location_by_id,
        room_routes::create_room,
        room_routes::get_rooms,
        room_routes::get_room_by_id,
        room_routes::patch_room_by_id,
        room_routes::delete_room_by_id,
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
        attendee::AttendeePatch,
        permissions::Permission,
        roles::Role,
        auth::LoginRequest,
        location::Location,
        location::LocationPatch,
        room::Room,
        room::RoomPatch,
    ))
)]

pub struct ApiDoc;
