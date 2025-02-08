use crate::{omni_error::OmniError, setup::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use tracing::{error};
use utoipa::ToSchema;
use uuid::Uuid;
use super::utils::{handle_failed_to_get_resource_by_id};


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
    responses((
    status=200, description = "Ok",
    body=Vec<Tournament>,
    example=json!(get_tournaments_list_example())
)))]
async fn get_tournaments(State(state): State<AppState>) -> Response {
    match query_as!(Tournament, "SELECT * FROM tournaments")
        .fetch_all(&state.connection_pool)
        .await
    {
        Ok(tournaments) => Json(tournaments).into_response(),
        Err(e) => {
            error!("Error getting a list of tournaments: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Create a new tournament
#[utoipa::path(
    post,
    request_body=Tournament,
    path = "/tournament",
    responses((
        status=200, 
        description = "Tournament created successfully",
        body=Tournament,
        example=json!(get_tournament_example_with_id())
    ))
)]
async fn create_tournament(
    State(state): State<AppState>,
    Json(json): Json<Tournament>,
) -> Response {
    let pool = &state.connection_pool;

    match Tournament::post(json, pool).await {
        Ok(tournament) => Json(tournament).into_response(),
        Err(e) => {
            error!("Error creating a new tournament: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Get details of an existing tournament
#[utoipa::path(get, path = "/tournament/{id}", 
    responses((status=200, description = "Ok", body=Tournament,
    example=json!
    (get_tournament_example_with_id())
    )),
    params(("id", description = "Tournament id"))
)]
async fn get_tournament_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    match Tournament::get_by_id(id, &state.connection_pool).await {
        Ok(tournament) => Json(tournament).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Patch an existing tournament
#[utoipa::path(patch, path = "/tournament/{id}", 
    request_body=TournamentPatch,
    params(("id", description = "Tournament id")),
    responses(
        (
            status=200, description = "Tournament patched successfully",
            body=Tournament,
            example=json!(get_tournament_example_with_id())
        )
    )
)]
async fn patch_tournament_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(new_tournament): Json<TournamentPatch>,
) -> Response {
    let pool = &state.connection_pool;
    match Tournament::get_by_id(id, pool).await {
    Ok(tournament) => match tournament.patch(new_tournament, pool).await {
        Ok(patched_tournament) => Json(patched_tournament).into_response(),
        Err(e) => {
            error!("Error patching a tournament with id {}: {e}", id);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        },
    },
    Err(e) => handle_failed_to_get_resource_by_id(e),
    }
}


/// Delete an existing tournament.
/// 
/// This operation is only allowed when there are no resources (i.e. teams, roles, and debates)
/// referencing this tournament.
#[utoipa::path(delete, path = "/tournament/{id}", 
    responses(
        (status=204, description = "Tournament deleted successfully"),
        (status=404, description = "Tournament not found"),
        (status=409, description = r#"
            There are teams, roles, or debates that
            reference this tournament. They must be deleted first
        "#)
    ),
    params(("id", description = "Tournament id"))
)]
async fn delete_tournament_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    match Tournament::get_by_id(id, &state.connection_pool).await {
        Ok(tournament) => match tournament.delete(&state.connection_pool).await {
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
            Err(e) =>
            {
                if e.is_sqlx_foreign_key_violation() {
                    return OmniError::DependentResourcesError.into_response()
                }
                else {
                    error!("Error deleting a tournament with id {id}: {e}");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            },
        },
        Err(e) => return handle_failed_to_get_resource_by_id(e),
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
