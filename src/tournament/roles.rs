use std::fmt;

use serde::{Deserialize, Serialize};
use sqlx::{query, Pool, Postgres};
use strum::VariantArray;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{omni_error::OmniError, users::permissions::Permission};

#[derive(Debug, PartialEq, Deserialize, ToSchema, VariantArray, Clone, Serialize)]
/// Within a tournament, users must be granted roles for their
/// permissions to be defined. Each role comes with a predefined
/// set of permissions to perform certain operations.
/// By default, a newly created user has no roles.
/// Multiple users can have the same role.
pub enum Role {
    /// This role grants all possible permissions within a tournament.
    Organizer,
    /// Judges can submit their verdicts regarding debates they were assigned to.
    Judge,
    /// Marshalls are responsible for conducting debates.
    /// For pragmatic reasons, they can submit verdicts on Judges' behalf.
    Marshall,
}

impl Role {
    pub fn get_role_permissions(&self) -> Vec<Permission> {
        use Permission as P;
        match self {
            Role::Organizer => P::VARIANTS.to_vec(),
            Role::Judge => vec![
                P::ReadAttendees,
                P::ReadDebates,
                P::ReadTeams,
                P::ReadTournament,
                P::SubmitOwnVerdictVote,
            ],
            Role::Marshall => vec![
                P::ReadDebates,
                P::ReadAttendees,
                P::ReadTeams,
                P::ReadTournament,
                P::SubmitVerdict,
            ],
        }
    }

    pub async fn post(
        user_id: Uuid,
        tournament_id: Uuid,
        roles: Vec<Role>,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Role>, OmniError> {
        let _ = tournament_id;
        let roles_as_strings = Role::roles_vec_to_string_array(&roles);
        match query!(
            r#"INSERT INTO roles(id, user_id, tournament_id, roles)
            VALUES ($1, $2, $3, $4) RETURNING roles"#,
            Uuid::now_v7(),
            user_id,
            tournament_id,
            &roles_as_strings
        )
        .fetch_one(pool)
        .await
        {
            Ok(record) => {
                let string_vec = record.roles.unwrap();
                let mut created_roles: Vec<Role> = vec![];
                for role_string in string_vec {
                    created_roles.push(Role::try_from(role_string)?);
                }
                return Ok(created_roles);
            }
            Err(e) => Err(e)?,
        }
    }

    pub fn roles_vec_to_string_array(roles: &Vec<Role>) -> Vec<String> {
        let mut string_vec = vec![];
        for role in roles {
            string_vec.push(role.to_string());
        }
        return string_vec;
    }

    pub async fn patch(
        user_id: Uuid,
        tournament_id: Uuid,
        roles: Vec<Role>,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Role>, OmniError> {
        let roles_as_strings = Role::roles_vec_to_string_array(&roles);
        match query!(
            r#"UPDATE roles SET roles = $1 WHERE user_id = $2 AND tournament_id = $3
            RETURNING roles"#,
            &roles_as_strings,
            user_id,
            tournament_id
        )
        .fetch_one(pool)
        .await
        {
            Ok(record) => {
                let string_vec = record.roles.unwrap();
                let mut created_roles: Vec<Role> = vec![];
                for role_string in string_vec {
                    created_roles.push(Role::try_from(role_string)?);
                }
                return Ok(created_roles);
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn delete(
        user_id: Uuid,
        tournament_id: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<(), OmniError> {
        match query!(
            r"DELETE FROM roles WHERE user_id = $1 AND tournament_id = $2",
            user_id,
            tournament_id
        )
        .execute(pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
}

impl TryFrom<&str> for Role {
    type Error = OmniError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Organizer" => Ok(Role::Organizer),
            "Marshall" => Ok(Role::Marshall),
            "Judge" => Ok(Role::Judge),
            _ => Err(OmniError::RolesParsingError),
        }
    }
}

impl TryFrom<String> for Role {
    type Error = OmniError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Organizer" => Ok(Role::Organizer),
            "Marshall" => Ok(Role::Marshall),
            "Judge" => Ok(Role::Judge),
            _ => Err(OmniError::RolesParsingError),
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::Organizer => write!(f, "Organizer"),
            Role::Judge => write!(f, "Judge"),
            Role::Marshall => write!(f, "Marshall"),
        }
    }
}
