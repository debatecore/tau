use crate::setup::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sqlx::types::JsonValue;
use sqlx::{query, query_as, Error, Pool, Postgres};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Tournament {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    id: Uuid,
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
    ) -> Result<Tournament, Error> {
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
                error!("Error creating a tournament: {e}");
                Err(e)
                // TO-DO: Handle duplicate full names
            }
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Tournament, Error> {
        match query_as!(Tournament, "SELECT * FROM tournaments WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(tournament) => Ok(tournament),
            Err(e) => {
                error!("Error getting a tournament with id {id}: {e}");
                Err(e)
            }
        }
    }

    pub async fn patch(
        self,
        patch: TournamentPatch,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Tournament, Error> {
        let tournament = Tournament {
            id: self.id,
            full_name: patch.full_name.unwrap_or(self.full_name),
            shortened_name: patch.shortened_name.unwrap_or(self.shortened_name),
        };
        match query!(
            "UPDATE tournaments SET full_name = $2, shortened_name = $3 WHERE id = $1",
            tournament.id,
            tournament.full_name,
            tournament.shortened_name
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(tournament),
            Err(e) => {
                error!("Error updating a tournament with id {}: {e}", self.id);
                Err(e)
            }
        }
    }

    pub async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), Error> {
        match query!("DELETE FROM tournaments WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Error deleting a tournament with id {}: {e}", self.id);
                Err(e)
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

#[utoipa::path(get, path = "/tournament", 
    responses((
    status=200, description = "Ok",
    body=Vec<Tournament>,
    example=json!
    (
        [
        {
        "id": "c8594993-5be7-4273-a3ee-10d396e5dab0",
        "full_name": "Kórnik Debate League",
        "shortened_name": "KDL"
        },
        {
        "id": "83d5b28f-7024-4388-8bd2-c9f967a36f51",
        "full_name": "Poznań Debate Night",
        "shortened_name": "PDN"
        }
        ]
    )
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
#[utoipa::path(post, request_body=Tournament, path = "/tournament", responses((
    status=200, description = "Tournament created successfully",
    body=Tournament)
))]
async fn create_tournament(
    State(state): State<AppState>,
    Json(json): Json<Tournament>,
) -> Response {
    match Tournament::post(json, &state.connection_pool).await {
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
    ({
        "id": "c8594993-5be7-4273-a3ee-10d396e5dab0",
        "full_name": "Kórnik Debate League",
        "shortened_name": "KDL"
    })
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
            example=json!
            ({
                "id": "c8594993-5be7-4273-a3ee-10d396e5dab0",
                "full_name": "Kórnik Debate League",
                "shortened_name": "KDL"
            })
        )
    )
)]
async fn patch_tournament_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(tournament): Json<TournamentPatch>,
) -> Response {
    match Tournament::get_by_id(id, &state.connection_pool).await {
        Ok(existing_tournament) => match existing_tournament
            .patch(tournament, &state.connection_pool)
            .await
        {
            Ok(tournament) => Json(tournament).into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        /// TO-DO: handle a case in which the tournament does not exist in the first place
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Delete an existing tournament
#[utoipa::path(delete, path = "/tournament/{id}", 
    responses
    ((status=204, description = "Tournament deleted successfully")),
    params(("id", description = "Tournament id"))
)]
async fn delete_tournament_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    match Tournament::get_by_id(id, &state.connection_pool).await {
        Ok(tournament) => match tournament.delete(&state.connection_pool).await {
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        /// TO-DO: handle a case in which the tournament does not exist in the first place
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
