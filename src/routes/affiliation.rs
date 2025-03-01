use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Error, Pool, Postgres};
use tower_cookies::Cookies;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    omni_error::OmniError,
    setup::AppState,
    users::{permissions::Permission, roles::Role, TournamentUser, User},
};

use super::tournament::Tournament;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
/// Some Judges might be affiliated with certain teams,
/// which poses a risk of biased rulings.
/// Tournament Organizers can denote such affiliations.
/// A Judge is prevented from ruling debates wherein
/// one of the sides is a team they're affiliated with.
pub struct Affiliation {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    id: Uuid,
    tournament_id: Uuid,
    team_id: Uuid,
    judge_user_id: Uuid,
}

#[derive(Deserialize, ToSchema)]
pub struct AffiliationPatch {
    tournament_id: Option<Uuid>,
    team_id: Option<Uuid>,
    judge_user_id: Option<Uuid>,
}

impl Affiliation {
    async fn post(
        affiliation: Affiliation,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Affiliation, OmniError> {
        match query_as!(
            Affiliation,
            r#"INSERT INTO judge_team_assignments(id, judge_user_id, team_id, tournament_id)
            VALUES ($1, $2, $3, $4) RETURNING id, judge_user_id, team_id, tournament_id"#,
            affiliation.id,
            affiliation.judge_user_id,
            affiliation.team_id,
            affiliation.tournament_id
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(_) => Ok(affiliation),
            Err(e) => Err(e)?,
        }
    }

    async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Affiliation, Error> {
        match query_as!(
            Affiliation,
            "SELECT * FROM judge_team_assignments WHERE id = $1",
            id
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(affiliation) => Ok(affiliation),
            Err(e) => Err(e),
        }
    }

    async fn patch(
        self,
        patch: Affiliation,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Affiliation, Error> {
        match query!(
            "UPDATE judge_team_assignments SET judge_user_id = $1, tournament_id = $2, team_id = $3 WHERE id = $4",
            patch.judge_user_id,
            patch.tournament_id,
            patch.team_id,
            self.id,
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(patch),
            Err(e) => Err(e),
        }
    }

    async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), Error> {
        match query!("DELETE FROM judge_team_assignments WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn validate(&self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        let user = User::get_by_id(self.judge_user_id, pool).await?;
        if !user.has_role(Role::Judge, self.tournament_id, pool).await? {
            return Err(OmniError::NotAJudgeError);
        }

        let _tournament = Tournament::get_by_id(self.tournament_id, pool).await?;

        if self.already_exists(pool).await? {
            return Err(OmniError::ResourceAlreadyExistsError);
        }

        Ok(())
    }

    async fn already_exists(&self, pool: &Pool<Postgres>) -> Result<bool, OmniError> {
        match query_as!(Affiliation,
            "SELECT * FROM judge_team_assignments WHERE judge_user_id = $1 AND tournament_id = $2 AND team_id = $3",
            self.judge_user_id,
            self.tournament_id,
            self.team_id
        ).fetch_optional(pool).await {
            Ok(result) => {
                if result.is_none() {
                    return Ok(false);
                }
                else {
                    return Ok(true);
                }
            },
            Err(e) => Err(e)?,
        }
    }
}

pub fn route() -> Router<AppState> {
    Router::new()
        .route(
            "/affiliation",
            get(get_affiliations).post(create_affiliation),
        )
        .route(
            "/affiliation/:affiliation_id",
            get(get_affiliation_by_id)
                .patch(patch_affiliation_by_id)
                .delete(delete_affiliation_by_id),
        )
}

/// Create a new affiliation
///
/// Available only to Organizers and the infrastructure admin.
#[utoipa::path(post, request_body=Affiliation, path = "/user/{user_id}/tournament/{tournament_id}/affiliation",
    responses
    (
        (
            status=200, description = "Affiliation created successfully",
            body=Affiliation,
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403,
            description = "The user is not permitted to modify affiliations within this tournament"
        ),
        (status=404, description = "Tournament or affiliation not found"),
        (status=500, description = "Internal server error"),
    )
)]
async fn create_affiliation(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(affiliation): Json<Affiliation>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteAffiliations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    affiliation.validate(pool).await?;
    match Affiliation::post(affiliation, pool).await {
        Ok(affiliation) => Ok(Json(affiliation).into_response()),
        Err(e) => {
            error!("Error creating a new affiliation: {e}");
            Err(e)
        }
    }
}

#[utoipa::path(get, path = "/user/{user_id}/tournament/{tournament_id}/affiliation",
    responses
    (
        (status=200, description = "Ok", body=Vec<Affiliation>),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403,
            description = "The user is not permitted to read affiliations within this tournament"
        ),
        (status=404, description = "Tournament or affiliation not found"),
        (status=500, description = "Internal server error"),
    )
)]
/// Get a list of all user affiliations.
///
/// Available only to Organizers and the infrastructure admin.
async fn get_affiliations(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(user_id): Path<Uuid>,
    Path(tournament_id): Path<Uuid>,
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
        return Err(OmniError::NotAJudgeError);
    }

    let _tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match query_as!(
        Affiliation,
        "SELECT * FROM judge_team_assignments WHERE tournament_id = $1",
        tournament_id
    )
    .fetch_all(&state.connection_pool)
    .await
    {
        Ok(affiliations) => Ok(Json(affiliations).into_response()),
        Err(e) => {
            error!(
                "Error getting affiliations of user {} within tournament {}: {e}",
                user_id, tournament_id
            );
            Err(e)?
        }
    }
}

/// Get details of an existing affiliation
///
/// Available only to Organizers and the infrastructure admin.
#[utoipa::path(get, path = "/user/{user_id}/tournament/{tournament_id}/affiliation/{id}",
    responses(
        (status=200, description = "Ok", body=Affiliation),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403,
            description = "The user is not permitted to read affiliations within this tournament"
        ),
        (status=404, description = "Tournament or affiliation not found"),
        (status=500, description = "Internal server error"),
    ),
)]
async fn get_affiliation_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadAffiliations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    match Affiliation::get_by_id(id, pool).await {
        Ok(affiliation) => Ok(Json(affiliation).into_response()),
        Err(e) => {
            error!("Error getting a affiliation with id {id}: {e}");
            Err(e)?
        }
    }
}

/// Patch an existing affiliation
///
/// Available only to Organizers and the infrastructure admin.
#[utoipa::path(patch, path = "/user/{user_id}/tournament/{tournament_id}/affiliation/{id}",
    request_body=Affiliation,
    responses(
        (status=200, description = "Affiliation patched successfully", body=Affiliation),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403,
            description = "The user is not permitted to modify affiliations within this tournament"
        ),
        (status=404, description = "Tournament or affiliation not found"),
        (
            status=409,
            description = "This affiliation already exists",
        ),
        (status=500, description = "Internal server error"),
    )
)]
async fn patch_affiliation_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(new_affiliation): Json<AffiliationPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
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
        tournament_id: new_affiliation
            .tournament_id
            .unwrap_or(old_affiliation.tournament_id),
        team_id: new_affiliation.team_id.unwrap_or(old_affiliation.team_id),
    };
    new_affiliation.validate(pool).await?;

    match old_affiliation.patch(new_affiliation, pool).await {
        Ok(affiliation) => Ok(Json(affiliation).into_response()),
        Err(e) => Err(e)?,
    }
}

/// Delete an existing affiliation
///
/// Available only to Organizers and the infrastructure admin.
#[utoipa::path(delete, path = "/user/{user_id}/tournament/{tournament_id}/affiliation/{id}",
    responses
    (
        (status=204, description = "Affiliation deleted successfully"),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403,
            description = "The user is not permitted to modify affiliations within this tournament"
        ),
        (status=404, description = "Tournament or affiliation not found"),
    ),
)]
async fn delete_affiliation_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteAffiliations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let affiliation = Affiliation::get_by_id(id, pool).await?;
    match affiliation.delete(&state.connection_pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => {
            error!("Error deleting a affiliation with id {id}: {e}");
            Err(e)?
        }
    }
}
