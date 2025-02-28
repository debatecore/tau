use core::fmt;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use sqlx::{query, query_as, Pool, Postgres};
use strum::VariantArray;
use tower_cookies::Cookies;
use tracing::error;
use uuid::Uuid;

use crate::{
    omni_error::OmniError,
    setup::AppState,
    users::{permissions::Permission, TournamentUser, User},
};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

pub fn route() -> Router<AppState> {
    Router::new().route(
        "/user/:user_id/tournament/:tournament_id/roles",
        post(create_user_roles)
            .get(get_user_roles)
            .patch(patch_user_roles)
            .delete(delete_user_roles),
    )
}

/// Grant Roles to a user.
///
/// Available only to Organizers and and the infrastructure admin.
#[utoipa::path(
    post,
    request_body=Vec<Role>,
    path = "/user/{user_id}/tournament/{tournament_id}/roles",
    responses(
        (
        status=200, description = "Roles created successfully",
        body=Vec<Role>
        ),
        (status=400, description = "Bad request"),
        (
            status=401,
            description = "The user is not permitted to modify roles within this tournament"
        ),
        (status=404, description = "User of tournament not found"),
        (status=409, description = "The user is already granted roles within this tournament. Use PATCH method to modify user roles"),
        (status=500, description = "Internal server error"),
    )
)]
async fn create_user_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((user_id, tournament_id)): Path<(Uuid, Uuid)>,
    Json(json): Json<Vec<Role>>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::WriteRoles) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    let user_to_be_granted_roles = User::get_by_id(user_id, pool).await?;
    let roles = user_to_be_granted_roles
        .get_roles(tournament_id, pool)
        .await?;
    if !roles.is_empty() {
        return Err(OmniError::ResourceAlreadyExistsError);
    }

    match Role::post(user_id, tournament_id, json, pool).await {
        Ok(role) => Ok(Json(role).into_response()),
        Err(e) => {
            error!(
                "Error creating roles for user {} within tournament {}: {e}",
                user_id, tournament_id
            );
            Err(e)?
        }
    }
}

/// Get roles a user is given within a tournament
///
/// The user must be given a role within this tournament to use this endpoint.
#[utoipa::path(get, path = "/user/{user_id}/tournament/{tournament_id}/roles",
    responses(
        (status=200, description = "Ok", body=Vec<Role>,
            example=json!(get_roles_example())
        ),
        (status=400, description="Bad request"),
        (status=401, description="The user is not permitted to see roles, meaning they don't have any role within this tournament"),
        (status=404, description="User or tournament not found"),
        (status=500, description="Internal server error"),
    ),
)]
async fn get_user_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((user_id, tournament_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.roles.is_empty() {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    let requested_user = User::get_by_id(user_id, pool).await?;
    match requested_user.get_roles(tournament_id, pool).await {
        Ok(roles) => Ok(Json(roles).into_response()),
        Err(e) => {
            error!(
                "Error getting roles of user {} within tournament {}: {e}",
                user_id, tournament_id
            );
            Err(e)?
        }
    }
}

/// Overwrite roles a user is given within a tournament
///
/// Available only to the tournament Organizers and the infrastructure admin.
#[utoipa::path(patch, path = "/tournament/{tournament_id}/role/{id}",
    request_body=Vec<Role>,
    responses(
        (
            status=200, description = "Roles patched successfully",
            body=Vec<Role>,
            example=json!(get_roles_example())
        ),
        (status=400, description = "Bad request"),
        (
            status=401,
            description = "The user is not permitted to modify roles within this tournament"
        ),
        (status=404, description = "Tournament or user not found, or the user has not been assigned any roles yet"),
        (status=500, description = "Internal server error"),
    )
)]
async fn patch_user_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((user_id, tournament_id)): Path<(Uuid, Uuid)>,
    Json(new_roles): Json<Vec<Role>>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::WriteRoles) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    let modified_user = TournamentUser::get_by_id(user_id, tournament_id, pool).await?;
    if modified_user.roles.is_empty() {
        return Err(OmniError::ResourceNotFoundError);
    }

    match Role::patch(user_id, tournament_id, new_roles, pool).await {
        Ok(roles) => Ok(Json(roles).into_response()),
        Err(e) => {
            error!(
                "Error patching roles of user {} within tournament {}: {e}",
                user_id, tournament_id
            );
            Err(e)
        }
    }
}

/// Delete user roles within a tournament
/// This operation effectively means banning the user from a tournament.
/// Available only to the tournament Organizers and the infrastructure admin.
#[utoipa::path(delete, path = "/tournament/{tournament_id}/role/{id}",
    responses
    (
        (status=204, description = "Role deleted successfully"),
        (status=400, description = "Bad request"),
        (
            status=401,
            description = "The user is not permitted to modify roles within this tournament"
        ),
        (status=404, description = "User or tournament not found"),
        (status=500, description = "Internal server error"),
    ),

)]
async fn delete_user_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((user_id, tournament_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::WriteRoles) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    match Role::delete(user_id, tournament_id, pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => {
            error!(
                "Error deleting roles of user {} within tournament {}: {e}",
                user_id, tournament_id
            );
            Err(e)?
        }
    }
}

fn get_roles_example() -> String {
    r#"
        ["Marshall", "Judge"]
    "#
    .to_owned()
}

#[test]
fn role_to_string() {
    let judge = Role::Judge;
    let marshall = Role::Marshall;
    let organizer = Role::Organizer;

    assert!(judge.to_string() == "Judge");
    assert!(marshall.to_string() == "Marshall");
    assert!(organizer.to_string() == "Organizer")
}

#[test]
fn role_vecs_to_string() {
    let roles = Role::VARIANTS.to_vec();
    let roles_count = roles.len();
    let roles_as_strings = Role::roles_vec_to_string_array(&roles);
    for i in 0..roles_count {
        assert!(roles_as_strings[i] == roles[i].to_string())
    }
}

#[test]
fn string_to_roles() {
    let role_strings = vec!["Marshall", "Judge", "Organizer", "Gżdacz"];

    let marshall_role = Role::try_from(role_strings[0]).unwrap();
    let judge_role = Role::try_from(role_strings[1]).unwrap();
    let organizer_role = Role::try_from(role_strings[2]).unwrap();
    let fake_role = Role::try_from(role_strings[3]);

    assert!(marshall_role == Role::Marshall);
    assert!(judge_role == Role::Judge);
    assert!(organizer_role == Role::Organizer);
    assert!(fake_role.is_err());
}
