use axum::Router;
use chrono::Utc;
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::auth;
use crate::routes::user_routes;
use crate::setup::AppState;

use crate::routes::affiliation_routes;
use crate::routes::attendee_routes;
use crate::routes::debate_routes;
use crate::routes::location_routes;
use crate::routes::motion_routes;
use crate::routes::phase_routes;
use crate::routes::roles_routes;
use crate::routes::room_routes;
use crate::routes::round_routes;
use crate::routes::team_routes;
use crate::routes::tournament_routes;
use crate::tournament;
use crate::tournament::affiliation;
use crate::tournament::attendee;
use crate::tournament::debate;
use crate::tournament::location;
use crate::tournament::motion;
use crate::tournament::phase;
use crate::tournament::roles;
use crate::tournament::room;
use crate::tournament::round;
use crate::tournament::team;
use crate::users;
use crate::users::permissions;
use crate::users::photourl;

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
        auth::auth_clear,
        user_routes::get_users,
        user_routes::create_user,
        user_routes::get_user_by_id,
        user_routes::patch_user_by_id,
        user_routes::delete_user_by_id,
        roles_routes::create_user_roles,
        roles_routes::get_user_roles,
        roles_routes::patch_user_roles,
        roles_routes::delete_user_roles,
        affiliation_routes::create_affiliation,
        affiliation_routes::get_affiliations,
        affiliation_routes::get_affiliation_by_id,
        affiliation_routes::patch_affiliation_by_id,
        affiliation_routes::delete_affiliation_by_id,
        round_routes::create_round,
        round_routes::get_round_by_id,
        round_routes::get_rounds,
        round_routes::patch_round_by_id,
        round_routes::delete_round_by_id,
        phase_routes::create_phase,
        phase_routes::get_phase_by_id,
        phase_routes::get_phases,
        phase_routes::patch_phase_by_id,
        phase_routes::delete_phase_by_id
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
        users::UserWithPassword,
        users::UserPatch,
        users::User,
        photourl::PhotoUrl,
        affiliation::Affiliation,
        affiliation::AffiliationPatch,
        phase::Phase,
        phase::PhasePatch,
        phase::PhaseStatus,
        round::Round,
        round::RoundPatch,
        round::RoundStatus,
    ))
)]

pub struct ApiDoc;
