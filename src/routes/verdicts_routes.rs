use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use reqwest::StatusCode;
use sqlx::query_as;
use tower_cookies::Cookies;
use tracing::error;
use uuid::Uuid;

use crate::{
    omni_error::OmniError,
    setup::AppState,
    tournaments::verdicts::{Verdict, VerdictPatch},
    users::{permissions::Permission, TournamentUser},
};

pub fn route() -> Router<AppState> {
    Router::new()
        .route(
            "/tournaments/{tournament_id}/debates/{debate_id}/verdicts",
            post(create_verdict).get(get_verdicts),
        )
        .route(
            "/tournaments/{tournament_id}/debates/{debate_id}/verdicts/{verdict_id}",
            get(get_verdict_by_id)
                .patch(patch_verdict_by_id)
                .delete(delete_verdict_by_id),
        )
}

/// Create a new verdict
///
/// Requires SubmitOwnVerdictVote permission. Available to Judges, Organizers and the admin.
#[utoipa::path(post, request_body=Verdict, path = "/tournaments/{tournament_id}/debates/{debate_id}/verdicts",
    responses(
        (status=200, description = "Ok", body=Verdict),
        (status=400, description = "Bad request"),
        (status=401, description = "Unauthorized"),
        (status=404, description = "Resource not found"),
        (status=422, description = "Unprocessable entity"),
        (status=500, description = "Internal server error"),
    ),
    tag="verdicts"
)]
async fn create_verdict(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((tournament_id, _debate_id)): Path<(Uuid, Uuid)>,
    Json(verdict): Json<Verdict>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::SubmitOwnVerdictVote) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    match Verdict::post(verdict, pool).await {
        Ok(verdict) => Ok(Json(verdict).into_response()),
        Err(e) => {
            error!("Error creating a new verdict: {e}");
            Err(e)
        }
    }
}

#[utoipa::path(get, path = "/tournaments/{tournament_id}/debates/{debate_id}/verdicts",
    responses
    (
        (status=200, description = "Ok", body=Vec<Verdict>),
        (status=400, description = "Bad request"),
        (status=401, description = "Unauthorized"),
        (status=404, description = "Resource not found"),
        (status=422, description = "Unprocessable entity"),
        (status=500, description = "Internal server error"),
    ),
    tag="verdicts"
)]
/// Get a list of all verdicts regarding a given debate
///
/// The user must be granted any role within the corresponding tournament to use this endpoint.
async fn get_verdicts(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((tournament_id, debate_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match query_as!(
        Verdict,
        "SELECT * FROM verdicts WHERE debate_id = $1",
        debate_id
    )
    .fetch_all(&state.connection_pool)
    .await
    {
        Ok(verdicts) => Ok(Json(verdicts).into_response()),
        Err(e) => {
            error!("Error getting verdicts of debate {}: {e}", debate_id);
            Err(e)?
        }
    }
}

/// Get details of an existing verdict
///
/// The user must be granted any role within the corresponding tournament to use this endpoint.
#[utoipa::path(get, path = "/tournaments/{tournament_id}/debates/{debate_id}/verdicts/{verdict_id}",
    responses(
        (status=200, description = "Ok", body=Verdict),
        (status=400, description = "Bad request"),
        (status=401, description = "Unauthorized"),
        (status=404, description = "Resource not found"),
        (status=422, description = "Unprocessable entity"),
        (status=500, description = "Internal server error"),
    ),
    tag="verdicts"
)]
async fn get_verdict_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((tournament_id, _debate_id, id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let verdict = Verdict::get_by_id(id, pool).await?;
    TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    Ok(Json(verdict).into_response())
}
///
/// Requires SubmitOwnVerdictVote permission to update your own verdicts.
/// Requires SubmitVerdict permission to change the judge or correct verdicts on behalf of others.
/// Available to Judges, Organizers and the admin.
/// The judge specified in the verdict must have either SubmitOwnVerdictVote or SubmitVerdict permission.
/// Attempting to change the verdict's judge without SubmitVerdict permission will result in 401.
/// Patch an existing verdict
#[utoipa::path(patch, path = "/tournaments/{tournament_id}/debates/{debate_id}/verdicts/{verdict_id}",
    request_body=Verdict,
    responses(
        (status=200, description = "Ok", body=Verdict),
        (status=400, description = "Bad request"),
        (status=401, description = "Unauthorized"),
        (status=404, description = "Resource not found"),
        (status=422, description = "Unprocessable entity"),
        (status=500, description = "Internal server error"),
    ),
    tag="verdicts"
)]
#[axum::debug_handler]
async fn patch_verdict_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((tournament_id, _debate_id, id)): Path<(Uuid, Uuid, Uuid)>,
    Json(new_verdict): Json<VerdictPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;

    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::SubmitOwnVerdictVote) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let old_verdict = Verdict::get_by_id(id, pool).await?;

    let new_verdict = Verdict {
        id: old_verdict.id,
        judge_user_id: new_verdict
            .judge_user_id
            .unwrap_or(old_verdict.judge_user_id),
        debate_id: new_verdict.debate_id.unwrap_or(old_verdict.debate_id),
        proposition_won: new_verdict
            .proposition_won
            .unwrap_or(old_verdict.proposition_won),
    };

    // Check if trying to change the judge_user_id
    if old_verdict.judge_user_id != new_verdict.judge_user_id {
        // Only users with SubmitVerdict permission can change who the judge is
        if !tournament_user.has_permission(Permission::SubmitVerdict) {
            return Err(OmniError::ExplicitError {
                status: StatusCode::UNAUTHORIZED,
                message:
                    "Correcting verdicts can only be conducted by judges who made them"
                        .to_string(),
            });
        }
    }

    // Verify the judge user exists and has appropriate permissions
    let judge_tournament_user =
        TournamentUser::get_by_id(new_verdict.judge_user_id, tournament_id, pool).await?;

    if !judge_tournament_user.has_permission(Permission::SubmitOwnVerdictVote)
        && !judge_tournament_user.has_permission(Permission::SubmitVerdict)
    {
        return Err(OmniError::ExplicitError {
            status: StatusCode::UNAUTHORIZED,
            message: "The specified judge does not have permission to submit verdicts"
                .to_string(),
        });
    }

    // Check if the new verdict would be a duplicate of an existing one (except for the current one)
    if new_verdict.already_exists(pool).await? {
        return Err(OmniError::ResourceAlreadyExistsError);
    }

    match old_verdict.patch(new_verdict, pool).await {
        Ok(verdict) => Ok(Json(verdict).into_response()),
        Err(e) => Err(e)?,
    }
}

/// Delete an existing verdict
///
/// Requires SubmitOwnVerdictVote permission. Available to Judges, Organizers and the admin.
#[utoipa::path(delete, path = "/tournaments/{tournament_id}/debates/{debate_id}/verdicts/{verdict_id}",
    responses(
        (status=204, description = "No content"),
        (status=400, description = "Bad request"),
        (status=401, description = "Unauthorized"),
        (status=404, description = "Resource not found"),
        (status=422, description = "Unprocessable entity"),
        (status=500, description = "Internal server error"),
    ),
    tag="verdicts"
)]
async fn delete_verdict_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((tournament_id, _debate_id, id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;

    let verdict = Verdict::get_by_id(id, pool).await?;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::SubmitOwnVerdictVote) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    match verdict.delete(&state.connection_pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => {
            error!("Error deleting a verdict with id {id}: {e}");
            Err(e)?
        }
    }
}
