use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use strum::VariantArray;
use tower_cookies::Cookies;
use tracing::error;
use uuid::Uuid;

use crate::{
    omni_error::OmniError,
    setup::AppState,
    tournament::roles::Role,
    users::{permissions::Permission, TournamentUser, User},
};

pub fn route() -> Router<AppState> {
    Router::new().route(
        "/user/:user_id/tournament/:tournament_id/roles",
        post(create_user_roles)
            .get(get_user_roles)
            .patch(patch_user_roles)
            .delete(delete_user_roles),
    )
}

/// Grant roles to a user
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
    ),
    tag = "roles"
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

/// List roles a user is given within a tournament
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
    tag = "roles"
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
#[utoipa::path(patch, path = "/user/{user_id}/tournament/{tournament_id}/roles",
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
    ),
    tag = "roles"
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
#[utoipa::path(delete, path = "/user/{user_id}/tournament/{tournament_id}/roles",
    responses
    (
        (status=204, description = "Roles deleted successfully"),
        (status=400, description = "Bad request"),
        (
            status=401,
            description = "The user is not permitted to modify roles within this tournament"
        ),
        (status=404, description = "User or tournament not found"),
        (status=500, description = "Internal server error"),
    ),
    tag = "roles"
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
