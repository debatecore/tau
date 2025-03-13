use crate::{omni_error::OmniError, setup::AppState, tournament::roles::Role, users::{photourl::PhotoUrl, User}};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use rand::rngs::OsRng;
use serde::Deserialize;
use sqlx::{query, Pool, Postgres};
use tower_cookies::Cookies;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use super::tournament::Tournament;

#[derive(Deserialize, ToSchema)]
pub struct UserPatch {
    pub handle: Option<String>,
    pub picture_link: Option<PhotoUrl>,
    pub password: Option<String>,
}


#[derive(Clone, Deserialize, ToSchema)]
pub struct UserWithPassword {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    pub handle: String,
    pub picture_link: Option<PhotoUrl>,
    pub password: String,
}

impl User {
    pub async fn get_by_id(user_id: Uuid, pool: &Pool<Postgres>) -> Result<User, OmniError> {
        let user =
            sqlx::query!("SELECT handle, picture_link FROM users WHERE id = $1", user_id)
                .fetch_one(pool)
                .await?;

        Ok(User {
            id: user_id,
            handle: user.handle,
            picture_link: match user.picture_link {
                Some(url) => Some(PhotoUrl::new(&url)?),
                None => None,
            },
        })
    }

    pub async fn get_by_handle(
        handle: &str,
        pool: &Pool<Postgres>,
    ) -> Result<User, OmniError> {
        let user = sqlx::query!(
            "SELECT id, picture_link FROM users WHERE handle = $1",
            handle
        )
        .fetch_one(pool)
        .await?;

        Ok(User {
            id: user.id,
            handle: handle.to_string(),
            picture_link: match user.picture_link {
                Some(url) => Some(PhotoUrl::new(&url)?),
                None => None,
            },
        })
    }

    pub async fn get_all(pool: &Pool<Postgres>) -> Result<Vec<User>, OmniError> {
        let users = sqlx::query!("SELECT id, handle, picture_link FROM users")
            .fetch_all(pool)
            .await?
            .iter()
            .map(|u| {
                Ok(User {
                    id: u.id,
                    handle: u.handle.clone(),
                    picture_link: match u.picture_link.clone() {
                        Some(url) => Some(PhotoUrl::new(&url)?),
                        None => None,
                    },
                })
            })
            .collect::<Result<Vec<User>, OmniError>>()?;
        Ok(users)
    }
    
    pub async fn post(
        user: User,
        password: String,
        pool: &Pool<Postgres>,
    ) -> Result<User, OmniError> {
        let pic = match &user.picture_link {
            Some(url) => Some(url.as_str()),
            None => None,
        };
        let hash = User::generate_password_hash(&password).unwrap();
        match sqlx::query!(
            "INSERT INTO users VALUES ($1, $2, $3, $4)",
            &user.id,
            &user.handle,
            pic,
            hash
        )
        .execute(pool)
        .await
        {
            Ok(_) => Ok(user),
            Err(e) => Err(e)?,
        }
    }


    pub async fn patch(
        self,
        patch: UserPatch,
        pool: &Pool<Postgres>,
    ) -> Result<User, OmniError> {
        let picture_link = match &patch.picture_link {
            Some(url) => Some(url.clone()),
            None => self.picture_link.clone(),
        };
        let updated_user = User {
            id: self.id,
            handle: patch.handle.clone().unwrap_or(self.handle.clone()),
            picture_link
        };
        if patch.password != None {
            self.update_user_and_change_password(&patch, pool).await?;
        }
        self.update_user_without_changing_password(&patch, pool).await?;
        Ok(updated_user)
    }

    async fn update_user_and_change_password(&self, patch: &UserPatch, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        let picture_link = match &patch.picture_link {
            Some(url) => Some(url.as_url().to_string()),
            None => Some(self.picture_link.as_ref().unwrap().as_str().to_owned()),
        };
        let password_hash = User::generate_password_hash(&patch.password.as_ref().unwrap()).unwrap().clone();
        match query!("UPDATE users SET handle = $1, picture_link = $2, password_hash = $3 WHERE id = $4",
            patch.handle,
            picture_link,
            password_hash,
            self.id
        ).execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
    
    fn generate_password_hash(password: &str) -> Result<String, OmniError> {
        let hash = {
            let argon = Argon2::default();
            let salt = SaltString::generate(&mut OsRng);
            match argon.hash_password(password.as_bytes(), &salt) {
                Ok(hash) => hash.to_string(),
                Err(e) => return Err(e)?,
            }
        };
        Ok(hash)
    }

    async fn update_user_without_changing_password(&self, patch: &UserPatch, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        let picture_link = match &patch.picture_link {
            Some(url) => Some(url.as_url().to_string()),
            None => Some(self.picture_link.as_ref().unwrap().as_str().to_owned()),
        };
        match query!("UPDATE users SET handle = $1, picture_link = $2 WHERE id = $3",
            patch.handle,
            picture_link,
            self.id
        ).execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
    

    pub async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("DELETE FROM users WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(e)?
            }
        }
    }

    // ---------- DATABASE HELPERS ----------
    pub async fn get_roles(
        &self,
        tournament_id: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Role>, OmniError> {
        let roles_result = sqlx::query!(
            "SELECT roles FROM roles WHERE user_id = $1 AND tournament_id = $2",
            self.id,
            tournament_id
        )
        .fetch_optional(pool)
        .await?;

        if roles_result.is_none() {
            return Ok(vec![]);
        }

        let roles_strings = roles_result.unwrap().roles.unwrap();
        let mut roles_vec = vec![];
        for role_string in roles_strings {
            roles_vec.push(Role::try_from(role_string)?);
        }
        Ok(roles_vec)
    }

    pub async fn is_organizer_of_any_tournament(&self, pool: &Pool<Postgres>) -> Result<bool, OmniError> {
        let tournaments = Tournament::get_all(pool).await?;
        for tournament in tournaments {
            let roles = self.get_roles(tournament.id, pool).await?;
            if roles.contains(&Role::Organizer) {
                return Ok(true);
            }
        }
        return Ok(false);
        
    }

    pub async fn invalidate_all_sessions(&self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("DELETE FROM sessions WHERE user_id = $1", self.id).execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
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

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/user", get(get_users).post(create_user))
        .route(
            "/user/:id",
            get(get_user_by_id)
                .delete(delete_user_by_id)
                .patch(patch_user_by_id),
        )
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
))]
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
    )
)]
async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Json(json): Json<UserWithPassword>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let user = User::authenticate(&headers, cookies, &pool).await?;
    if !user.is_infrastructure_admin() && !user.is_organizer_of_any_tournament(pool).await? {
        return Err(OmniError::UnauthorizedError);
    }

    let user_without_password = User::from(json.clone());
    match User::post(user_without_password, json.password, pool).await {
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
/// Available to the infrastructure admin and the user modifying their own account.
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
    )
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
        Ok(patched_user) => Ok(Json(patched_user).into_response()),
        Err(e) => {
            error!("Error patching a user with id {}: {e}", id);
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
