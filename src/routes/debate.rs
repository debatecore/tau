use crate::setup::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, query_as, Error, Pool, Postgres};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Debate {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    id: Uuid,
    motion_id: Uuid,
    marshall_user_id: Uuid,
}

#[serde_inline_default]
#[derive(Deserialize, ToSchema)]
pub struct DebatePatch {
    motion_id: Option<Uuid>,
    marshall_user_id: Option<Uuid>,
}

impl Debate {
    pub async fn post(
        debate: Debate,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Debate, Error> {
        match query_as!(
            Debate,
            r#"INSERT INTO debates(id, motion_id, marshall_user_id)
            VALUES ($1, $2, $3) RETURNING id, motion_id, marshall_user_id"#,
            debate.id,
            debate.motion_id,
            debate.marshall_user_id
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(_) => Ok(debate),
            Err(e) => {
                error!("Error creating a debate: {e}");
                Err(e)
            }
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Debate, Error> {
        match query_as!(Debate, "SELECT * FROM debates WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(debate) => Ok(debate),
            Err(e) => {
                error!("Error getting a debate with id {id}: {e}");
                Err(e)
            }
        }
    }

    pub async fn patch(
        self,
        patch: DebatePatch,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Debate, Error> {
        let debate = Debate {
            id: self.id,
            motion_id: patch.motion_id.unwrap_or(self.motion_id),
            marshall_user_id: patch.marshall_user_id.unwrap_or(self.marshall_user_id),
        };
        match query!(
            "UPDATE debates SET motion_id = $1, marshall_user_id = $2 WHERE id = $3",
            debate.motion_id,
            debate.marshall_user_id,
            debate.id,
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(debate),
            Err(e) => {
                error!("Error patching a debate with id {}: {e}", self.id);
                Err(e)
            }
        }
    }

    pub async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), Error> {
        match query!("DELETE FROM debates WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Error deleting a debate with id {}: {e}", self.id);
                Err(e)
            }
        }
    }
}

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/debate", get(get_debates).post(create_debate))
        .route(
            "/debate/:id",
            get(get_debate_by_id)
                .delete(delete_debate_by_id)
                .patch(patch_debate_by_id),
        )
}

#[utoipa::path(get, path = "/debate", 
    responses((
        status=200, description = "Ok",
        body=Vec<Debate>,
    ))
)]
/// Get a list of all debates
async fn get_debates(State(state): State<AppState>) -> Response {
    match query_as!(Debate, "SELECT * FROM debates")
        .fetch_all(&state.connection_pool)
        .await
    {
        Ok(debates) => Json(debates).into_response(),
        Err(e) => {
            error!("Error getting a list of debates: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Create a new debate
#[utoipa::path(post, request_body=Debate, path = "/debate",
    responses((
        status=200,
        description = "Debate created successfully",
        body=Debate,
    ))
)]
async fn create_debate(
    State(state): State<AppState>,
    Json(json): Json<Debate>,
) -> Response {
    // TO-DO: Ensure that the new debate name is unique within a tournament
    match Debate::post(json, &state.connection_pool).await {
        Ok(debate) => Json(debate).into_response(),
        Err(e) => {
            error!("Error creating a new debate: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Get details of an existing debate
#[utoipa::path(get, path = "/debate/{id}", 
    responses((
        status=200,
        description = "Ok",
        body=Debate,
    )),
    params(("id", description = "Debate id"))
)]
async fn get_debate_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    match Debate::get_by_id(id, &state.connection_pool).await {
        Ok(debate) => Json(debate).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Patch an existing debate
#[utoipa::path(patch, path = "/debate/{id}", 
    request_body=DebatePatch,
    params(("id", description = "Debate id")),
    responses(
        (
            status=200, description = "Debate patched successfully",
            body=Debate,
        )
    )
)]
async fn patch_debate_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(new_debate): Json<DebatePatch>,
) -> Response {
    // TO-DO: Ensure that the new debate name is unique within a tournament
    match Debate::get_by_id(id, &state.connection_pool).await {
        Ok(existing_debate) => match existing_debate
            .patch(new_debate, &state.connection_pool)
            .await
        {
            Ok(patched_debate) => Json(patched_debate).into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        // TO-DO: handle a case in which the debate does not exist in the first place
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Delete an existing debate
#[utoipa::path(delete, path = "/debate/{id}", 
    responses
    ((status=204, description = "Debate deleted successfully")),
    params(
        ("id", description = "Debate id"),
    )
)]
async fn delete_debate_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    match Debate::get_by_id(id, &state.connection_pool).await {
        Ok(debate) => match debate.delete(&state.connection_pool).await {
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        // TO-DO: handle a case in which the debate does not exist in the first place
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
