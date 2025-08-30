use crate::{omni_error::OmniError, setup::AppState, users::{photourl::PhotoUrl, UserPatch, User}};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, patch},
    Json, Router,
};
use serde::Deserialize;
use tower_cookies::Cookies;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;



#[derive(Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UserWithPassword {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    pub handle: String,
    pub picture_link: Option<PhotoUrl>,
    pub password: String,
}

impl From<UserWithPassword> for User {
    fn from(value: UserWithPassword) -> Self {
        User {
            id: value.id,
            handle: value.handle,
            picture_link: value.picture_link
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UserPasswordPatch {
    pub new_password: String,
}

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/user", get(get_users).post(create_user))
        .route(
            "/user/:id",
            get(get_user_by_id)
                .delete(delete_user_by_id)
                .patch(patch_user_by_id),
        )
        .route("/user/:id/password", patch(change_user_password))
}

/// Get a list of all users
/// 
/// This request only returns the users the user is permitted to see.
/// The user must be given any role within a user to see it.
#[utoipa::path(get, path = "/user", 
    responses(
        (
            status=200, description = "Ok",
            body=Vec<User>,
            example=json!(get_users_list_example())
        ),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "Authentication error"
        ),
        (status=500, description = "Internal server error")
    ),
    tag = "user"
)]
async fn get_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    User::authenticate(&headers, cookies, pool).await?;

    match User::get_all(pool).await {
        Ok(users) => Ok(Json(users).into_response()),
        Err(e) => {
            error!("Error listing users: {e}");
            Err(e)?
        }
    }
}

/// Create a new user
/// 
/// Available to the infrastructure admin and tournament Organizers.
#[utoipa::path(
    post,
    request_body=User,
    path = "/user",
    responses
    (
        (
            status=200, 
            description = "User created successfully",
            body=User,
            example=json!(get_user_example_with_id())
        ),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to create users"
        ),
        (status=404, description = "User not found"),
        (status=422, description = "Invalid picture link"),
        (status=500, description = "Internal server error")
    ),
    tag = "user"
)]
async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Json(json): Json<UserWithPassword>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let user = User::authenticate(&headers, cookies, &pool).await?;
    if !user.is_infrastructure_admin() && !user.can_create_users_within_any_tournament(pool).await? {
        return Err(OmniError::UnauthorizedError);
    }

    let user_without_password = User::from(json.clone());
    match User::create(user_without_password, json.password, pool).await {
        Ok(user) => Ok(Json(user).into_response()),
        Err(e) => {
            error!("Error creating a new user: {e}");
            Err(e)?
        }
    }
}

/// Get details of an existing user
/// 
/// Every user is permitted to use this endpoint.
#[utoipa::path(get, path = "/user/{id}", 
    responses
    (
        (
            status=200, description = "Ok", body=User,
            example=json!
            (get_user_example_with_id())
        ),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "Authentication error"
        ),
        (status=404, description = "User not found"),
        (status=500, description = "Internal server error")
    ),
    tag = "user"
)]
async fn get_user_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    User::authenticate(&headers, cookies, pool).await?;

    match User::get_by_id(id, pool).await {
        Ok(user) => Ok(Json(user).into_response()),
        Err(e) => {
            error!("Error getting a user with id {}: {e}", id);
            Err(e)
        }
    }
}

/// Patch an existing user
/// 
/// Allows to modify user data not related to security.
/// Available to the infrastructure admin and the user modifying their own account.
/// In order to change user password, use the /user/{id}/password endpoint.
#[utoipa::path(patch, path = "/user/{id}", 
    request_body=UserPatch,
    responses(
        (
            status=200, description = "User patched successfully",
            body=User,
            example=json!(get_user_example_with_id())
        ),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to modify this user"
        ),
        (status=404, description = "User not found"),
        (status=409, description = "A user with this name already exists"),
        (status=422, description = "Invalid picture link"),
        (status=500, description = "Internal server error")
    ),
    tag = "user"
)]
async fn patch_user_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Json(new_user): Json<UserPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let requesting_user =
        User::authenticate(&headers, cookies, &pool).await?;

    let user_to_be_patched = User::get_by_id(id, pool).await?;
    
    match requesting_user.is_infrastructure_admin() || requesting_user.id == user_to_be_patched.id {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    match user_to_be_patched.patch(new_user, pool).await {
        Ok(patched_user) => Ok(Json(patched_user as User).into_response()),
        Err(e) => {
            error!("Error patching a user with id {}: {e}", id);
            Err(e)?
        }
    }
}

/// Change user password
/// 
/// Available to the infrastructure admin and the user modifying their own account.
#[utoipa::path(patch, path = "/user/{id}/password", 
    request_body=UserPasswordPatch,
    responses(
        (
            status=200, description = "User password changed successfully",
        ),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to modify this user"
        ),
        (status=404, description = "User not found"),
        (status=500, description = "Internal server error")
    ),
    tag = "user"
)]
async fn change_user_password(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Json(password_patch): Json<UserPasswordPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let requesting_user =
        User::authenticate(&headers, cookies, &pool).await?;

    let user_to_be_patched = User::get_by_id(id, pool).await?;
    
    match requesting_user.is_infrastructure_admin() || requesting_user.id == user_to_be_patched.id {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    match user_to_be_patched.change_password(&password_patch.new_password, pool).await {
        Ok(()) => Ok(StatusCode::OK.into_response()),
        Err(e) => {
            error!("Error changing password of a user with id {}: {e}", id);
            Err(e)?
        }
    }
}


/// Delete an existing user.
/// 
/// Available only to the infrastructure admin,
/// who's account cannot be deleted.
/// Deleted user is automatically logged out of all sessions.
/// This operation is only allowed when there are no resources
/// referencing this user.
#[utoipa::path(delete, path = "/user/{id}", 
    responses(
        (status=204, description = "User deleted successfully"),
        (status=400, description = "Bad request"),
        (status=401, description = "The user is not permitted to delete this user"),
        (status=404, description = "User not found"),
        (status=409, description = "Other resources reference this user. They must be deleted first")
    ),
    tag = "user"
)]
async fn delete_user_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let requesting_user =
        User::authenticate(&headers, cookies, pool).await?;

    match requesting_user.is_infrastructure_admin() {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    let user_to_be_deleted = User::get_by_id(id, pool).await?;

    match user_to_be_deleted.is_infrastructure_admin() {
        true => return Err(OmniError::UnauthorizedError),
        false => ()
    }

    user_to_be_deleted.invalidate_all_sessions(pool).await?;
    match user_to_be_deleted.delete(pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) =>
        {
            if e.is_sqlx_foreign_key_violation() {
                return Err(OmniError::DependentResourcesError)
            }
            else {
                error!("Error deleting a user with id {id}: {e}");
                return Err(e)?;
            }
        },
    }
}

fn get_user_example_with_id() -> String {
    r#"
    {
        "id": "01941265-8b3c-733f-a6ae-075c079f2f81",
        "handle": "jmanczak",
        "picture_link": "https://placehold.co/128x128.png"
    }
    "#
    .to_owned()
}

fn get_users_list_example() -> String {
    r#"
        [
        {
            "id": "01941265-8b3c-733f-a6ae-075c079f2f81",
            "handle": "jmanczak",
            "picture_link": "https://placehold.co/128x128.png"
        },
        {
            "id": "01941265-8b3c-733f-a6ae-091c079c2921",
            "handle": "Matthew Goodman",
            "picture_link": "https://placehold.co/128x128.png"
        }
        ]
    "#.to_owned()
}
