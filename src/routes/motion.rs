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
pub struct Motion {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    id: Uuid,
    motion: String,
    adinfo: Option<String>,
}

#[serde_inline_default]
#[derive(Deserialize, ToSchema)]
pub struct MotionPatch {
    motion: Option<String>,
    #[serde_inline_default(None)]
    adinfo: Option<String>,
}

impl Motion {
    pub async fn post(
        motion: Motion,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Motion, Error> {
        match query_as!(
            Motion,
            r#"INSERT INTO motions(id, motion, adinfo)
        VALUES ($1, $2, $3) RETURNING id, motion, adinfo"#,
            motion.id,
            motion.motion,
            motion.adinfo
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(_) => Ok(motion),
            Err(e) => {
                error!("Error creating a motion: {e}");
                Err(e)
                // TO-DO: Handle duplicate motions
            }
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Motion, Error> {
        match query_as!(Motion, "SELECT * FROM motions WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(motion) => Ok(motion),
            Err(e) => {
                error!("Error getting a motion with id {id}: {e}");
                Err(e)
            }
        }
    }

    pub async fn patch(
        self,
        patch: MotionPatch,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Motion, Error> {
        let motion = Motion {
            id: self.id,
            motion: patch.motion.unwrap_or(self.motion),
            adinfo: patch.adinfo,
        };
        match query!(
            "UPDATE motions SET motion = $1, adinfo = $2 WHERE id = $3",
            motion.motion,
            motion.adinfo,
            motion.id,
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(motion),
            Err(e) => {
                error!("Error patching a motion with id {}: {e}", self.id);
                Err(e)
            }
        }
    }

    pub async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), Error> {
        match query!("DELETE FROM motions WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Error deleting a motion with id {}: {e}", self.id);
                Err(e)
            }
        }
    }
}

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/motion", get(get_motions).post(create_motion))
        .route(
            "/motion/:id",
            get(get_motion_by_id)
                .delete(delete_motion_by_id)
                .patch(patch_motion_by_id),
        )
}

#[utoipa::path(get, path = "/motion", 
    responses((
    status=200, description = "Ok",
    body=Vec<Motion>,
    example=json!
    ([
        {
        "id": "c8594993-5be7-4273-a3ee-10d396e5dab0",
        "motion": "This House Would abolish the UN Security Council.",
        },
        {
        "id": "83d5b28f-7024-4388-8bd2-c9f967a36f51",
        "motion": "As a society of a newly established state, we would opt for a representative democracy system.",
        "adinfo": r#"In the middle of the Baltic Sea, an island with a population has appeared. The new state of \"Balticstan\" is seeking the best political system to govern itself. The country has guaranteed independence and is sovereign over regional powers at the time of the debate. Balticstan represents the maximum average of all countries bordering the Baltic Sea (nine countries in total) regarding population, economy, problems and opportunities."#
        }
    ])
)))]
async fn get_motions(State(state): State<AppState>) -> Response {
    match query_as!(Motion, "SELECT * FROM motions")
        .fetch_all(&state.connection_pool)
        .await
    {
        Ok(motions) => Json(motions).into_response(),
        Err(e) => {
            error!("Error getting a list of motions: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Create a new motion
#[utoipa::path(post, request_body=Motion, path = "/motion", responses((
    status=200, description = "Motion created successfully",
    body=Motion)
))]
async fn create_motion(
    State(state): State<AppState>,
    Json(json): Json<Motion>,
) -> Response {
    // TO-DO: Ensure that the new motion name is unique within a tournament
    match Motion::post(json, &state.connection_pool).await {
        Ok(motion) => Json(motion).into_response(),
        Err(e) => {
            error!("Error creating a new motion: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Get details of an existing motion
#[utoipa::path(get, path = "/motion/{id}", 
    responses((status=200, description = "Ok", body=Motion,
    example=json!
    ({
        "id": "83d5b28f-7024-4388-8bd2-c9f967a36f51",
        "motion": "As a society of a newly established state, we would opt for a representative democracy system.",
        "adinfo": r#"In the middle of the Baltic Sea, an island with a population has appeared. The new state of \"Balticstan\" is seeking the best political system to govern itself. The country has guaranteed independence and is sovereign over regional powers at the time of the debate. Balticstan represents the maximum average of all countries bordering the Baltic Sea (nine countries in total) regarding population, economy, problems and opportunities."#
    })
    )),
    params(("id", description = "Motion id"))
)]
async fn get_motion_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    match Motion::get_by_id(id, &state.connection_pool).await {
        Ok(motion) => Json(motion).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Patch an existing motion
#[utoipa::path(patch, path = "/motion/{id}", 
    request_body=MotionPatch,
    params(("id", description = "Motion id")),
    responses(
        (
            status=200, description = "Motion patched successfully",
            body=Motion,
            example=json!
            ({
                "id": "83d5b28f-7024-4388-8bd2-c9f967a36f51",
                "motion": "As a society of a newly established state, we would opt for a representative democracy system.",
                "adinfo": r#"In the middle of the Baltic Sea, an island with a population has appeared. The new state of \"Balticstan\" is seeking the best political system to govern itself. The country has guaranteed independence and is sovereign over regional powers at the time of the debate. Balticstan represents the maximum average of all countries bordering the Baltic Sea (nine countries in total) regarding population, economy, problems and opportunities."#
            })
        )
    )
)]
async fn patch_motion_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(new_motion): Json<MotionPatch>,
) -> Response {
    // TO-DO: Ensure that the new motion name is unique within a tournament
    match Motion::get_by_id(id, &state.connection_pool).await {
        Ok(existing_motion) => match existing_motion
            .patch(new_motion, &state.connection_pool)
            .await
        {
            Ok(patched_motion) => Json(patched_motion).into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        // TO-DO: handle a case in which the motion does not exist in the first place
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Delete an existing motion
#[utoipa::path(delete, path = "/motion/{id}", 
    responses
    ((status=204, description = "Motion deleted successfully")),
    params(("id", description = "Motion id"))
)]
async fn delete_motion_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    match Motion::get_by_id(id, &state.connection_pool).await {
        Ok(motion) => match motion.delete(&state.connection_pool).await {
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        // TO-DO: handle a case in which the motion does not exist in the first place
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
