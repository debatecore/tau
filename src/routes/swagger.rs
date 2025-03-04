use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::auth;
use crate::setup::AppState;

use crate::routes::attendee;
use crate::routes::debate;
use crate::routes::location;
use crate::routes::motion;
use crate::routes::room;
use crate::routes::team;
use crate::routes::tournament;
use crate::tournament_impl;
use crate::tournament_impl::attendee_impl;
use crate::tournament_impl::debate_impl;
use crate::tournament_impl::location_impl;
use crate::tournament_impl::motion_impl;
use crate::tournament_impl::room_impl;
use crate::tournament_impl::team_impl;
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
        attendee::delete_attendee_by_id,
        auth::auth_login,
        location::create_location,
        location::get_locations,
        location::get_location_by_id,
        location::patch_location_by_id,
        location::delete_location_by_id,
        room::create_room,
        room::get_rooms,
        room::get_room_by_id,
        room::patch_room_by_id,
        room::delete_room_by_id,
    ),
    components(schemas(
        version::VersionDetails,
        version::VersionBits,
        version::GitInfo,
        tournament_impl::Tournament,
        tournament_impl::TournamentPatch,
        motion_impl::Motion,
        motion_impl::MotionPatch,
        team_impl::Team,
        team_impl::TeamPatch,
        debate_impl::Debate,
        debate_impl::DebatePatch,
        attendee_impl::Attendee,
        attendee_impl::AttendeePatch,
        permissions::Permission,
        roles::Role,
        auth::LoginRequest,
        location_impl::Location,
        location_impl::LocationPatch,
        room_impl::Room,
        room_impl::RoomPatch,
    ))
)]

pub struct ApiDoc;
