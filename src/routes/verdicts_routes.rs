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
    tournaments::{
        debates::Debate,
        verdicts::{Verdict, VerdictPatch},
    },
    users::{permissions::Permission, TournamentUser},
};

pub fn route() -> Router<AppState> {
    Router::new()
        .route(
            "/tournaments/:tournament_id/debates/:debate_id/verdicts",
            post(create_verdict).get(get_verdicts),
        )
        // .route("/users/:user_id/verdicts/tournament/:tournament_id")
        .route(
            "/tournaments/:tournament_id/debates/:debate_id/verdicts/:verdict_id",
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
    Path((tournament_id, debate_id)): Path<(Uuid, Uuid)>,
    Json(verdict): Json<Verdict>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let debate = Debate::get_by_id(verdict.debate_id, pool).await?;
    let tournament_id = debate.tournament_id;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

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
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

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
    Path((_tournament_id, _debate_id, id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let verdict = Verdict::get_by_id(id, pool).await?;
    let tournament_id = verdict.infer_tournament_id(pool).await?;
    TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    Ok(Json(verdict).into_response())
}

/// Patch an existing verdict
///
/// Requires SubmitOwnVerdictVote permission. Available to Judges, Organizers and the admin.
#[utoipa::path(patch, path = "/users/{user_id}/verdicts/{id}",
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
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

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
    new_verdict.validate(tournament_id, pool).await?;

    match old_verdict.patch(new_verdict, pool).await {
        Ok(verdict) => Ok(Json(verdict).into_response()),
        Err(e) => Err(e)?,
    }
}

/// Delete an existing verdict
/// Requires SubmitOwnVerdictVote permission. Available to Judges, Organizers and the admin.
#[utoipa::path(delete, path = "/users/{user_id}/verdicts/{id}",
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
    Path((tournament_id, debate_id, id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;

    let verdict = Verdict::get_by_id(id, pool).await?;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

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
