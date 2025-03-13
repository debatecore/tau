use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::omni_error::OmniError;

use super::{room::Room, utils::get_optional_value_to_be_patched};

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
/// Some tournaments stretch across multiple locations.
/// This struct is intended to be a representation of a bigger location
/// (e.g. a particular school or university campus),
/// possibly containing multiple places (i.e. rooms)
/// to conduct debates at.
pub struct Location {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    /// Location name. Must be unique within a tournament.
    pub name: String,
    /// A field dedicated to store information about location address.
    /// While contents of this field could be included in remarks,
    /// its presence prompts the user to include address information.
    pub address: Option<String>,
    pub remarks: Option<String>,
    pub tournament_id: Uuid,
}

#[derive(ToSchema, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct LocationPatch {
    pub name: Option<String>,
    pub address: Option<String>,
    pub remarks: Option<String>,
    pub tournament_id: Option<Uuid>,
}

impl Location {
    pub async fn post(
        location: Location,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Location, OmniError> {
        match query_as!(
            Location,
            r#"INSERT INTO locations(id, name, address, remarks, tournament_id)
            VALUES ($1, $2, $3, $4, $5) RETURNING id, name, address, remarks, tournament_id"#,
            location.id,
            location.name,
            location.address,
            location.remarks,
            location.tournament_id
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(_) => Ok(location),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Location, OmniError> {
        match query_as!(Location, "SELECT * FROM locations WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(location) => Ok(location),
            Err(e) => Err(e)?,
        }
    }

    pub async fn patch(
        self,
        new_location: LocationPatch,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Location, OmniError> {
        let patch = Location {
            id: self.id,
            name: new_location.name.unwrap_or(self.name),
            address: get_optional_value_to_be_patched(self.address, new_location.address),
            remarks: get_optional_value_to_be_patched(self.remarks, new_location.remarks),
            tournament_id: new_location.tournament_id.unwrap_or(self.tournament_id),
        };
        match query!(
            r#"UPDATE locations set name = $1, address = $2,
            remarks = $3, tournament_id = $4
            WHERE id = $5"#,
            patch.name,
            patch.address,
            patch.remarks,
            patch.tournament_id,
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
        match query!("DELETE FROM locations WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_rooms(&self, pool: &Pool<Postgres>) -> Result<Vec<Room>, OmniError> {
        match query_as!(Room, "SELECT * FROM rooms WHERE location_id = $1", self.id)
            .fetch_all(pool)
            .await
        {
            Ok(rooms) => Ok(rooms),
            Err(e) => {
                error!("Error getting rooms of location {}: {e}", self.id);
                Err(e)?
            }
        }
    }
}
