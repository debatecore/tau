use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use sqlx::query;
use tower_cookies::Cookies;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    omni_error::OmniError,
    setup::AppState,
    tournaments::{
        debates::Debate,
        phases::Phase,
        rounds::Round,
        Tournament,
    },
    users::{permissions::Permission, TournamentUser},
};

#[derive(Serialize, ToSchema)]
struct TournamentLadderResponse {
    phases: Vec<Phase>,
    rounds: Vec<Round>,
    debates: Vec<Debate>,
}

pub fn route() -> Router<AppState> {
    Router::new().route("/tournament/{tournament_id}/ladder", get(get_ladder))
}

#[utoipa::path(get, path = "/tournament/{tournament_id}/ladder",
    responses(
        (status=200, description = "Ok", body=TournamentLadderResponse),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (status=403, description = "The user is not permitted to read this tournament ladder"),
        (status=404, description = "Tournament not found"),
        (status=500, description = "Internal server error")
    ),
    tag="tournaments"
)]
async fn get_ladder(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    if !tournament_user.has_permission(Permission::ReadPhases)
        || !tournament_user.has_permission(Permission::ReadRounds)
        || !tournament_user.has_permission(Permission::ReadDebates)
    {
        return Err(OmniError::InsufficientPermissionsError);
    }

    let _tournament = Tournament::get_by_id(tournament_id, pool).await?;

    let mut transaction = pool.begin().await?;
    query("SET TRANSACTION READ ONLY")
        .execute(&mut *transaction)
        .await?;

    let phases = Phase::get_all(tournament_id, &mut *transaction).await?;
    let rounds = Round::get_all(tournament_id, &mut *transaction).await?;
    let debates = Debate::get_all(tournament_id, &mut *transaction).await?;
    transaction.commit().await?;

    Ok(Json(TournamentLadderResponse {
        phases,
        rounds,
        debates,
    })
    .into_response())
}
