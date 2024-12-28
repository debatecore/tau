use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, query_as, Error, Pool, Postgres};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::setup::AppState;

#[serde_inline_default]
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Attendee {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    id: Uuid,
    name: String,
    position: Option<i32>,
    team_id: Option<Uuid>,
    #[serde_inline_default(0)]
    individual_points: i32,
    #[serde_inline_default(0)]
    penalty_points: i32,
}

#[derive(Deserialize, ToSchema)]
struct AttendeePatch {
    name: Option<String>,
    position: Option<i32>,
    team_id: Option<Uuid>,
    individual_points: Option<i32>,
    penalty_points: Option<i32>,
}

impl Attendee {
    async fn post(
        attendee: Attendee,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Attendee, Error> {
        match query_as!(
            Attendee,
            r#"INSERT INTO attendees
            (id, name, position, team_id, individual_points, penalty_points)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, position, team_id, individual_points, penalty_points"#,
            attendee.id,
            attendee.name,
            attendee.position,
            attendee.team_id,
            attendee.individual_points,
            attendee.penalty_points
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(attendee) => Ok(attendee),
            Err(e) => {
                error!("Error creating an attendee: {e}");
                Err(e)
            }
        }
    }

    async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Attendee, Error> {
        match query_as!(Attendee, "SELECT * FROM attendees WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(attendee) => Ok(attendee),
            Err(e) => {
                error!("Error getting an attendee by id {id}: e");
                Err(e)
            }
        }
    }

    async fn patch(
        self,
        connection_pool: &Pool<Postgres>,
        patch: AttendeePatch,
    ) -> Result<Attendee, Error> {
        let new_attendee = Attendee {
            id: self.id,
            name: patch.name.unwrap_or(self.name),
            position: patch.position,
            team_id: patch.team_id,
            individual_points: patch.individual_points.unwrap_or(self.individual_points),
            penalty_points: patch.penalty_points.unwrap_or(self.penalty_points),
        };
        match query!(
            r#"UPDATE attendees SET name = $1, position = $2, team_id = $3,
            individual_points = $4, penalty_points = $5 WHERE id = $6"#,
            new_attendee.name,
            new_attendee.position,
            new_attendee.team_id,
            new_attendee.individual_points,
            new_attendee.penalty_points,
            new_attendee.id
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(new_attendee),
            Err(e) => {
                error!("Error patching an attendee with id {}: e", self.id);
                Err(e)
            }
        }
    }

    async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), Error> {
        match query!("DELETE FROM attendees WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Error deleting an attendee with id {}: {e}", self.id);
                Err(e)
            }
        }
    }
}

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/attendee", post(create_attendee).get(get_attendees_list))
        .route(
            "/attendee/:id",
            get(get_attendee_by_id)
                .patch(patch_attendee_by_id)
                .delete(delete_attendee_by_id),
        )
}

async fn create_attendee(
    State(state): State<AppState>,
    Json(attendee): Json<Attendee>,
) -> Response {
    match Attendee::post(attendee, &state.connection_pool).await {
        Ok(attendee) => Json(attendee).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_attendees_list(State(state): State<AppState>) -> Response {
    match query_as!(Attendee, "SELECT * FROM attendees")
        .fetch_all(&state.connection_pool)
        .await
    {
        Ok(attendees) => Json(attendees).into_response(),
        Err(e) => {
            error!("Error getting a list of attendees: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_attendee_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    match Attendee::get_by_id(id, &state.connection_pool).await {
        Ok(attendee) => Json(attendee).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn patch_attendee_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(patch): Json<AttendeePatch>,
) -> Response {
    match Attendee::get_by_id(id, &state.connection_pool).await {
        Ok(existing_attendee) => {
            match existing_attendee.patch(&state.connection_pool, patch).await {
                Ok(new_attendee) => Json(new_attendee).into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn delete_attendee_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    match Attendee::get_by_id(id, &state.connection_pool).await {
        Ok(existing_attendee) => {
            match existing_attendee.delete(&state.connection_pool).await {
                Ok(_) => StatusCode::NO_CONTENT.into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
