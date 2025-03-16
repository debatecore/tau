use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::omni_error::OmniError;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
/// A debate must be held in a particular place (or Room).
/// A room must be assigned to a preexisting Location.
/// While a debate
pub struct Room {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    /// Must be unique within a location.
    pub name: String,
    pub remarks: Option<String>,
    pub location_id: Uuid,
}

#[derive(ToSchema, Deserialize)]
pub struct RoomPatch {
    pub name: Option<String>,
    pub remarks: Option<String>,
    pub location_id: Option<Uuid>,
}

impl Room {
    pub async fn post(
        room: Room,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Room, OmniError> {
        match query_as!(
            Room,
            r#"INSERT INTO rooms(id, name, remarks, location_id)
            VALUES ($1, $2, $3, $4) RETURNING id, name, remarks, location_id"#,
            room.id,
            room.name,
            room.remarks,
            room.location_id,
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(_) => Ok(room),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Room, OmniError> {
        match query_as!(Room, "SELECT * FROM rooms WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(room) => Ok(room),
            Err(e) => Err(e)?,
        }
    }

    pub async fn patch(
        self,
        new_room: RoomPatch,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Room, OmniError> {
        let patch = Room {
            id: self.id,
            name: new_room.name.unwrap_or(self.name),
            remarks: new_room.remarks.or(self.remarks),
            location_id: new_room.location_id.unwrap_or(self.location_id),
        };
        match query!(
            r#"UPDATE rooms set name = $1,
            remarks = $2, location_id = $3
            WHERE id = $4"#,
            patch.name,
            patch.remarks,
            patch.location_id,
            self.id,
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(patch),
            Err(e) => Err(e)?,
        }
    }

    pub async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("DELETE FROM rooms WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }

    pub async fn room_with_name_exists_in_location(
        name: &String,
        location_id: &Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<bool, OmniError> {
        match query!(
            "SELECT EXISTS(SELECT 1 FROM rooms WHERE name = $1 AND location_id = $2)",
            name,
            location_id
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(result) => Ok(result.exists.unwrap()),
            Err(e) => Err(e)?,
        }
    }
}
