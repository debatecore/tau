use crate::{omni_error::OmniError, setup::AppState, users::{permissions::Permission, TournamentUser, User}};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use tower_cookies::Cookies;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;


#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Tournament {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    id: Uuid,
    // Full name of the tournament. Must be unique.
    full_name: String,
    shortened_name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct TournamentPatch {
    full_name: Option<String>,
    shortened_name: Option<String>,
}

impl Tournament {
    pub async fn post(
        tournament: Tournament,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Tournament, OmniError> {
        match query_as!(
            Tournament,
            r#"INSERT INTO tournaments(id, full_name, shortened_name)
        VALUES ($1, $2, $3) RETURNING id, full_name, shortened_name"#,
            tournament.id,
            tournament.full_name,
            tournament.shortened_name
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(_) => Ok(tournament),
            Err(e) => {
                Err(e)?
            }
        }
    }

    pub async fn get_all(connection_pool: &Pool<Postgres>) -> Result<Vec<Tournament>, OmniError> {
        match query_as!(Tournament, "SELECT * FROM tournaments")
        .fetch_all(connection_pool)
        .await {
            Ok(tournaments) => Ok(tournaments),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Tournament, OmniError> {
        match query_as!(Tournament, "SELECT * FROM tournaments WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(tournament) => Ok(tournament),
            Err(e) => {
                error!("Error getting a tournament with id {id}: {e}");
                Err(e)?
            }
        }
    }

    pub async fn patch(
        self,
        patch: TournamentPatch,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Tournament, OmniError> {
        let tournament = Tournament {
            id: self.id,
            full_name: patch.full_name.unwrap_or(self.full_name),
            shortened_name: patch.shortened_name.unwrap_or(self.shortened_name),
        };
        match query!(
            "UPDATE tournaments SET full_name = $1, shortened_name = $2 WHERE id = $3",
            tournament.full_name,
            tournament.shortened_name,
            tournament.id,
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(tournament),
            Err(e) => {
                Err(e)?
            }
        }
    }

    pub async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("DELETE FROM tournaments WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(e)?
            }
        }
    }
}

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/tournament", get(get_tournaments).post(create_tournament))
        .route(
            "/tournament/:id",
            get(get_tournament_by_id)
                .delete(delete_tournament_by_id)
                .patch(patch_tournament_by_id),
        )
}

/// Get a list of all tournaments
#[utoipa::path(get, path = "/tournament", 
    responses(
        (
        status=200, description = "Ok",
        body=Vec<Tournament>,
        example=json!(get_tournaments_list_example())
        ),
        (
            status=403, 
            description = "The user is not permitted to read tournaments"
        ),
        (status=500, description = "Internal server error")
))]
async fn get_tournaments(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteDebates) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }
    match Tournament::get_all(&state.connection_pool).await
    {
        Ok(tournaments) => Ok(Json(tournaments).into_response()),
        Err(e) => {
            error!("Error getting a list of tournaments: {e}");
            Err(e)?
        }
    }
}

/// Create a new tournament
#[utoipa::path(
    post,
    request_body=Tournament,
    path = "/tournament",
    responses
    (
        (
            status=200, 
            description = "Tournament created successfully",
            body=Tournament,
            example=json!(get_tournament_example_with_id())
        ),
        (
            status=403, 
            description = "The user is not permitted to modify tournaments"
        ),
        (status=404, description = "Tournament not found"),
        (status=500, description = "Internal server error")
    )
)]
async fn create_tournament(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Json(json): Json<Tournament>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let user = User::authenticate(&headers, cookies, &pool).await?;
    if !user.is_infrastructure_admin() {
        return Err(OmniError::UnauthorizedError);
    }

    let tournament = Tournament::post(json, pool).await?;
    return Ok(Json(tournament).into_response());

    // TO-DO: make the user performing the operation admin within this tournament
}

/// Get details of an existing tournament
#[utoipa::path(get, path = "/tournament/{id}", 
    responses
    (
        (
            status=200, description = "Ok", body=Tournament,
            example=json!
            (get_tournament_example_with_id())
        ),
        (
            status=403, 
            description = "The user is not permitted to read tournaments"
        ),
        (status=404, description = "Tournament not found"),
        (status=500, description = "Internal server error")
    ),
)]
async fn get_tournament_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::ReadTournaments) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }
    match Tournament::get_by_id(id, pool).await {
        Ok(tournament) => Ok(Json(tournament).into_response()),
        Err(e) => Err(e),
    }
}

/// Patch an existing tournament
#[utoipa::path(patch, path = "/tournament/{id}", 
    request_body=TournamentPatch,
    responses(
        (
            status=200, description = "Tournament patched successfully",
            body=Tournament,
            example=json!(get_tournament_example_with_id())
        ),
        (
            status=403, 
            description = "The user is not permitted to modify tournaments"
        ),
        (status=404, description = "Tournament not found"),
        (status=409, description = "A tournament with this name already exists"),
        (status=500, description = "Internal server error")
    )
)]
async fn patch_tournament_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(new_tournament): Json<TournamentPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteTournaments) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    let tournament = Tournament::get_by_id(id, pool).await?;
    match tournament.patch(new_tournament, pool).await {
        Ok(patched_tournament) => Ok(Json(patched_tournament).into_response()),
        Err(e) => {
            error!("Error patching a tournament with id {}: {e}", id);
            Err(e)
        }
    }
}


/// Delete an existing tournament.
/// 
/// This operation is only allowed when there are no resources
/// referencing this tournament.
#[utoipa::path(delete, path = "/tournament/{id}", 
    responses(
        (status=204, description = "Tournament deleted successfully"),
        (status=403, description = "The user is not permitted to modify tournaments"),
        (status=404, description = "Tournament not found"),
        (status=409, description = "Other resources reference this tournament. They must be deleted first")
    ),
)]
async fn delete_tournament_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Response, OmniError> {
    match Tournament::get_by_id(id, &state.connection_pool).await {
        Ok(tournament) => match tournament.delete(&state.connection_pool).await {
            Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
            Err(e) =>
            {
                if e.is_sqlx_foreign_key_violation() {
                    return Err(OmniError::DependentResourcesError)
                }
                else {
                    error!("Error deleting a tournament with id {id}: {e}");
                    return Err(e)?;
                }
            },
        },
        Err(e) => return Err(e),
    }
}

fn get_tournament_example_with_id() -> String {
    r#"
    {
        "id": "01941265-8b3c-733f-a6ae-075c079f2f81",
        "full_name": "Kórnik Debate League",
        "shortened_name": "KDL"
    }
    "#
    .to_owned()
}

fn get_tournaments_list_example() -> String {
    r#"
        [
        {
        "id": "01941265-8b3c-733f-a6ae-075c079f2f81",
        "full_name": "Kórnik Debate League",
        "shortened_name": "KDL"
        },
        {
        "id": "01941265-507e-7987-b1ed-5c0f63ff6c6d",
        "full_name": "Poznań Debate Night",
        "shortened_name": "PND"
        }
        ]
    "#.to_owned()
}
