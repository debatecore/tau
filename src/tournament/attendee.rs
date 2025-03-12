use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, query_as, Pool, Postgres};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::omni_error::OmniError;

#[serde_inline_default]
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Attendee {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    pub name: String,
    /// Represents the attendee's position as a speaker
    /// (1 for the 1st speaker, 2 for the 2nd speaker, etc.).
    /// If the attendee is not a speaker, but is nonetheless
    /// affiliated with the team, the position should be None.
    /// Two attendees from the same team cannot be placed on the same position.
    pub position: Option<i32>,
    pub team_id: Uuid,
}

#[derive(Deserialize, ToSchema)]
pub struct AttendeePatch {
    pub name: Option<String>,
    pub position: Option<i32>,
    pub team_id: Option<Uuid>,
}

impl Attendee {
    pub async fn post(
        attendee: Attendee,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Attendee, OmniError> {
        match query_as!(
            Attendee,
            r#"INSERT INTO attendees
            (id, name, position, team_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, position, team_id"#,
            attendee.id,
            attendee.name,
            attendee.position,
            attendee.team_id,
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(attendee) => Ok(attendee),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Attendee, OmniError> {
        match query_as!(Attendee, "SELECT * FROM attendees WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(attendee) => Ok(attendee),
            Err(e) => Err(e)?,
        }
    }

    pub async fn patch(
        self,
        connection_pool: &Pool<Postgres>,
        patch: AttendeePatch,
    ) -> Result<Attendee, OmniError> {
        let new_attendee = Attendee {
            id: self.id,
            name: patch.name.unwrap_or(self.name),
            position: patch.position,
            team_id: patch.team_id.unwrap_or(self.team_id),
        };
        match query!(
            "UPDATE attendees SET name = $1, position = $2, team_id = $3 WHERE id = $4",
            new_attendee.name,
            new_attendee.position,
            new_attendee.team_id,
            new_attendee.id
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(new_attendee),
            Err(e) => Err(e)?,
        }
    }

    pub async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("DELETE FROM attendees WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
}
