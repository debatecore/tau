use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use sqlx::query_as;
use tower_cookies::Cookies;
use tracing::error;
use uuid::Uuid;

use crate::{
    omni_error::OmniError,
    setup::AppState,
    tournaments::{
        affiliations::{Affiliation, AffiliationPatch},
        roles::Role,
        teams::Team,
        Tournament,
    },
    users::{permissions::Permission, TournamentUser, User},
};

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/users/:user_id/affiliations", post(create_affiliation))
        .route(
            "/users/:user_id/affiliations/tournament/:tournament_id",
            get(get_affiliations),
        )
        .route(
            "/users/:user_id/affiliations/:id",
            get(get_affiliation_by_id)
                .patch(patch_affiliation_by_id)
                .delete(delete_affiliation_by_id),
        )
}

/// Create a new affiliation
///
/// Available only to Organizers and the infrastructure admin.
#[utoipa::path(post, request_body=Affiliation, path = "/users/{user_id}/affiliations",
    responses(
        (status=200, description = "Ok", body=Affiliation),
        (status=400, description = "Bad request"),
        (status=401, description = "Unauthorized"),
        (status=404, description = "Resource not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="affiliations"
)]
async fn create_affiliation(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(user_id): Path<Uuid>,
    Json(affiliation): Json<Affiliation>,
) -> Result<Response, OmniError> {
    if !params_and_affiliation_fields_match(&affiliation, &user_id) {
        return Err(OmniError::BadRequestError);
    }

    let pool = &state.connection_pool;
    let team = Team::get_by_id(affiliation.team_id, pool).await?;
    let tournament_id = team.tournament_id;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteAffiliations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    affiliation.validate(tournament_id, pool).await?;
    match Affiliation::post(affiliation, pool).await {
        Ok(affiliation) => Ok(Json(affiliation).into_response()),
        Err(e) => {
            error!("Error creating a new affiliation: {e}");
            Err(e)
        }
    }
}

fn params_and_affiliation_fields_match(
    affiliation: &Affiliation,
    user_id: &Uuid,
) -> bool {
    if !(&affiliation.judge_user_id == user_id) {
        return false;
    }
    return true;
}

#[utoipa::path(get, path = "/users/{user_id}/affiliations/tournament/{tournament_id}",
    responses
    (
        (status=200, description = "Ok", body=Vec<Affiliation>),
        (status=400, description = "Bad request"),
        (status=401, description = "Unauthorized"),
        (status=404, description = "Resource not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="affiliations"
)]
/// Get a list of all user affiliations within a given tournament.
///
/// Available only to Organizers and the infrastructure admin.
async fn get_affiliations(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((user_id, tournament_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadAffiliations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let affiliated_user = User::get_by_id(user_id, pool).await?;
    if !affiliated_user
        .has_role(Role::Judge, tournament_id, pool)
        .await?
    {
        return Err(OmniError::NotAJudgeAffiliationError);
    }

    let _tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match query_as!(Affiliation, "SELECT * FROM judge_team_assignments")
        .fetch_all(&state.connection_pool)
        .await
    {
        Ok(affiliations) => Ok(Json(affiliations).into_response()),
        Err(e) => {
            error!("Error getting affiliations of user {}: {e}", user_id);
            Err(e)?
        }
    }
}

/// Get details of an existing affiliation
///
/// Available only to Organizers and the infrastructure admin.
#[utoipa::path(get, path = "/users/{user_id}/affiliations/{id}",
    responses(
        (status=200, description = "Ok", body=Affiliation),
        (status=400, description = "Bad request"),
        (status=401, description = "Unauthorized"),
        (status=404, description = "Resource not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="affiliations"
)]
async fn get_affiliation_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((_user_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let affiliation = Affiliation::get_by_id(id, pool).await?;
    let tournament_id = affiliation.infer_tournament_id(pool).await?;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadAffiliations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    Ok(Json(affiliation).into_response())
}

/// Patch an existing affiliation
///
/// Available only to Organizers and the infrastructure admin.
#[utoipa::path(patch, path = "/users/{user_id}/affiliations/{id}",
    request_body=Affiliation,
    responses(
        (status=200, description = "Ok", body=Affiliation),
        (status=400, description = "Bad request"),
        (status=401, description = "Unauthorized"),
        (status=404, description = "Resource not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="affiliations"
)]
#[axum::debug_handler]
async fn patch_affiliation_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((_user_id, id)): Path<(Uuid, Uuid)>,
    Json(new_affiliation): Json<AffiliationPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;

    let affiliation = Affiliation::get_by_id(id, pool).await?;
    let team = Team::get_by_id(affiliation.team_id, pool).await?;
    let tournament_id = Tournament::get_by_id(team.tournament_id, pool).await?.id;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteAffiliations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let old_affiliation = Affiliation::get_by_id(id, pool).await?;

    let new_affiliation = Affiliation {
        id: old_affiliation.id,
        judge_user_id: new_affiliation
            .judge_user_id
            .unwrap_or(old_affiliation.judge_user_id),
        team_id: new_affiliation.team_id.unwrap_or(old_affiliation.team_id),
    };
    new_affiliation.validate(tournament_id, pool).await?;

    match old_affiliation.patch(new_affiliation, pool).await {
        Ok(affiliation) => Ok(Json(affiliation).into_response()),
        Err(e) => Err(e)?,
    }
}

/// Delete an existing affiliation
///
/// Available only to Organizers and the infrastructure admin.
#[utoipa::path(delete, path = "/users/{user_id}/affiliations/{id}",
    responses(
        (status=204, description = "No content"),
        (status=400, description = "Bad request"),
        (status=401, description = "Unauthorized"),
        (status=404, description = "Resource not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="affiliations"
)]
async fn delete_affiliation_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((_user_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;

    let affiliation = Affiliation::get_by_id(id, pool).await?;
    let tournament_id = affiliation.infer_tournament_id(pool).await?;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteAffiliations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    match affiliation.delete(&state.connection_pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => {
            error!("Error deleting a affiliation with id {id}: {e}");
            Err(e)?
        }
    }
}
