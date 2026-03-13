use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use sqlx::{query, Pool, Postgres};
use strum::VariantArray;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{omni_error::OmniError, users::permissions::Permission};

#[derive(Debug, PartialEq, Deserialize, ToSchema, VariantArray, Clone, Serialize)]
#[serde(deny_unknown_fields)]
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
    /// Marshals are responsible for conducting debates.
    /// For pragmatic reasons, they can submit verdicts on Judges' behalf.
    Marshal,
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
                P::ReadPhases,
                P::ReadRounds,
            ],
            Role::Marshal => vec![
                P::ReadDebates,
                P::ReadAttendees,
                P::ReadTeams,
                P::ReadTournament,
                P::ReadLocations,
                P::ReadRooms,
                P::SubmitVerdict,
                P::ReadPhases,
                P::ReadRounds,
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
        let roles_as_strings = roles.to_string_vec();
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
                let created_roles: Vec<Role> = string_vec
                    .into_iter()
                    .map(|role| Role::from_str(&role).unwrap())
                    .collect();
                return Ok(created_roles);
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn patch(
        user_id: Uuid,
        tournament_id: Uuid,
        roles: Vec<Role>,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Role>, OmniError> {
        let roles_as_strings = roles.to_string_vec();
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
                let created_roles = record
                    .roles
                    .unwrap()
                    .into_iter()
                    .map(|string| string.parse().unwrap())
                    .collect();
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

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::Organizer => write!(f, "Organizer"),
            Role::Judge => write!(f, "Judge"),
            Role::Marshal => write!(f, "Marshal"),
        }
    }
}

impl FromStr for Role {
    type Err = OmniError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Marshal" => Ok(Role::Marshal),
            "Judge" => Ok(Role::Judge),
            "Organizer" => Ok(Role::Organizer),
            _ => Err(OmniError::RolesParsingError),
        }
    }
}

pub trait RoleVecExt {
    fn to_string_vec(&self) -> Vec<String>;
}

impl RoleVecExt for Vec<Role> {
    fn to_string_vec(&self) -> Vec<String> {
        self.iter().map(|role| role.to_string()).collect()
    }
}
