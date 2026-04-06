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
        roles::Role,
        teams::Team,
        verdicts::{Verdict, VerdictPatch},
        Tournament,
    },
    users::{permissions::Permission, TournamentUser, User},
};

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/users/:user_id/verdicts", post(create_verdict))
        .route("/users/:user_id/verdicts/tournament/:tournament_id")
    // .route(
    //     "/users/:user_id/verdicts/:id",
    //     get(get_verdict_by_id)
    //         .patch(patch_verdict_by_id)
    //         .delete(delete_verdict_by_id),
    // )
}

/// Create a new verdict
///
/// Available only to Organizers and the infrastructure admin.
#[utoipa::path(post, request_body=Verdict, path = "/users/{user_id}/verdicts",
    responses(
        (status=200, description = "Ok", body=Verdict),
        (status=400, description = "Bad request"),
        (status=401, description = "Unauthorized"),
        (status=404, description = "Resource not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="verdicts"
)]
async fn create_verdict(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(user_id): Path<Uuid>,
    Json(verdict): Json<Verdict>,
) -> Result<Response, OmniError> {
    if !params_and_verdict_fields_match(&verdict, &user_id) {
        return Err(OmniError::BadRequestError);
    }

    let pool = &state.connection_pool;
    let team = Team::get_by_id(verdict.team_id, pool).await?;
    let tournament_id = team.tournament_id;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteVerdicts) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    verdict.validate(tournament_id, pool).await?;
    match Verdict::post(verdict, pool).await {
        Ok(verdict) => Ok(Json(verdict).into_response()),
        Err(e) => {
            error!("Error creating a new verdict: {e}");
            Err(e)
        }
    }
}

fn params_and_verdict_fields_match(verdict: &Verdict, user_id: &Uuid) -> bool {
    if !(&verdict.judge_user_id == user_id) {
        return false;
    }
    return true;
}

// #[utoipa::path(get, path = "/users/{user_id}/verdicts/tournament/{tournament_id}",
//     responses
//     (
//         (status=200, description = "Ok", body=Vec<Verdict>),
//         (status=400, description = "Bad request"),
//         (status=401, description = "Unauthorized"),
//         (status=404, description = "Resource not found"),
//         (status=500, description = "Internal server error"),
//     ),
//     tag="verdicts"
// )]
// /// Get a list of all user verdicts within a given tournament.
// ///
// /// Available only to Organizers and the infrastructure admin.
// async fn get_verdicts(
//     State(state): State<AppState>,
//     headers: HeaderMap,
//     cookies: Cookies,
//     Path((user_id, tournament_id)): Path<(Uuid, Uuid)>,
// ) -> Result<Response, OmniError> {
//     let pool = &state.connection_pool;
//     let tournament_user =
//         TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

//     match tournament_user.has_permission(Permission::ReadVerdicts) {
//         true => (),
//         false => return Err(OmniError::InsufficientPermissionsError),
//     }

//     let affiliated_user = User::get_by_id(user_id, pool).await?;
//     if !affiliated_user
//         .has_role(Role::Judge, tournament_id, pool)
//         .await?
//     {
//         return Err(OmniError::NotAJudgeVerdictError);
//     }

//     let _tournament = Tournament::get_by_id(tournament_id, pool).await?;
//     match query_as!(Verdict, "SELECT * FROM judge_team_assignments")
//         .fetch_all(&state.connection_pool)
//         .await
//     {
//         Ok(verdicts) => Ok(Json(verdicts).into_response()),
//         Err(e) => {
//             error!("Error getting verdicts of user {}: {e}", user_id);
//             Err(e)?
//         }
//     }
// }

// /// Get details of an existing verdict
// ///
// /// Available only to Organizers and the infrastructure admin.
// #[utoipa::path(get, path = "/users/{user_id}/verdicts/{id}",
//     responses(
//         (status=200, description = "Ok", body=Verdict),
//         (status=400, description = "Bad request"),
//         (status=401, description = "Unauthorized"),
//         (status=404, description = "Resource not found"),
//         (status=500, description = "Internal server error"),
//     ),
//     tag="verdicts"
// )]
// async fn get_verdict_by_id(
//     State(state): State<AppState>,
//     headers: HeaderMap,
//     cookies: Cookies,
//     Path((_user_id, id)): Path<(Uuid, Uuid)>,
// ) -> Result<Response, OmniError> {
//     let pool = &state.connection_pool;
//     let verdict = Verdict::get_by_id(id, pool).await?;
//     let tournament_id = verdict.infer_tournament_id(pool).await?;
//     let tournament_user =
//         TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

//     match tournament_user.has_permission(Permission::ReadVerdicts) {
//         true => (),
//         false => return Err(OmniError::InsufficientPermissionsError),
//     }

//     Ok(Json(verdict).into_response())
// }

// /// Patch an existing verdict
// ///
// /// Available only to Organizers and the infrastructure admin.
// #[utoipa::path(patch, path = "/users/{user_id}/verdicts/{id}",
//     request_body=Verdict,
//     responses(
//         (status=200, description = "Ok", body=Verdict),
//         (status=400, description = "Bad request"),
//         (status=401, description = "Unauthorized"),
//         (status=404, description = "Resource not found"),
//         (status=500, description = "Internal server error"),
//     ),
//     tag="verdicts"
// )]
// #[axum::debug_handler]
// async fn patch_verdict_by_id(
//     State(state): State<AppState>,
//     headers: HeaderMap,
//     cookies: Cookies,
//     Path((_user_id, id)): Path<(Uuid, Uuid)>,
//     Json(new_verdict): Json<VerdictPatch>,
// ) -> Result<Response, OmniError> {
//     let pool = &state.connection_pool;

//     let verdict = Verdict::get_by_id(id, pool).await?;
//     let team = Team::get_by_id(verdict.team_id, pool).await?;
//     let tournament_id = Tournament::get_by_id(team.tournament_id, pool).await?.id;
//     let tournament_user =
//         TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

//     match tournament_user.has_permission(Permission::WriteVerdicts) {
//         true => (),
//         false => return Err(OmniError::InsufficientPermissionsError),
//     }

//     let old_verdict = Verdict::get_by_id(id, pool).await?;

//     let new_verdict = Verdict {
//         id: old_verdict.id,
//         judge_user_id: new_verdict
//             .judge_user_id
//             .unwrap_or(old_verdict.judge_user_id),
//         team_id: new_verdict.team_id.unwrap_or(old_verdict.team_id),
//     };
//     new_verdict.validate(tournament_id, pool).await?;

//     match old_verdict.patch(new_verdict, pool).await {
//         Ok(verdict) => Ok(Json(verdict).into_response()),
//         Err(e) => Err(e)?,
//     }
// }

// /// Delete an existing verdict
// ///
// /// Available only to Organizers and the infrastructure admin.
// #[utoipa::path(delete, path = "/users/{user_id}/verdicts/{id}",
//     responses(
//         (status=204, description = "No content"),
//         (status=400, description = "Bad request"),
//         (status=401, description = "Unauthorized"),
//         (status=404, description = "Resource not found"),
//         (status=500, description = "Internal server error"),
//     ),
//     tag="verdicts"
// )]
// async fn delete_verdict_by_id(
//     State(state): State<AppState>,
//     headers: HeaderMap,
//     cookies: Cookies,
//     Path((_user_id, id)): Path<(Uuid, Uuid)>,
// ) -> Result<Response, OmniError> {
//     let pool = &state.connection_pool;

//     let verdict = Verdict::get_by_id(id, pool).await?;
//     let tournament_id = verdict.infer_tournament_id(pool).await?;
//     let tournament_user =
//         TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

//     match tournament_user.has_permission(Permission::SubmitOwnVerdictVote) {
//         true => (),
//         false => return Err(OmniError::InsufficientPermissionsError),
//     }

//     match verdict.delete(&state.connection_pool).await {
//         Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
//         Err(e) => {
//             error!("Error deleting a verdict with id {id}: {e}");
//             Err(e)?
//         }
//     }
// }
