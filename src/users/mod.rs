use crate::routes::roles::Role;
use axum::http::HeaderMap;
use permissions::Permission;
use photourl::PhotoUrl;
use serde::Serialize;
use sqlx::{Pool, Postgres};
use tower_cookies::Cookies;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::omni_error::OmniError;

pub mod auth;
pub mod infradmin;
pub mod permissions;
pub mod photourl;

#[derive(Serialize, Clone, ToSchema)]
pub struct User {
    pub id: Uuid,
    /// User handle used to log in and presented to other users.
    /// Must be unique.
    pub handle: String,
    /// A link to a profile picture. Accepted extensions are: png, jpg, jpeg, and webp.
    pub picture_link: Option<PhotoUrl>,
}

pub struct TournamentUser {
    pub user: User,
    pub roles: Vec<Role>,
}

impl TournamentUser {
    pub async fn authenticate(
        tournament_id: Uuid,
        headers: &HeaderMap,
        cookies: Cookies,
        pool: &Pool<Postgres>,
    ) -> Result<TournamentUser, OmniError> {
        let user = User::authenticate(headers, cookies, pool).await?;
        if user.is_infrastructure_admin() {
            return Ok(TournamentUser {
                user,
                roles: vec![],
            });
        }
        let roles = user.get_roles(tournament_id, pool).await?;
        return Ok(TournamentUser { user, roles });
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        if self.user.is_infrastructure_admin() {
            return true;
        } else {
            self.roles
                .iter()
                .any(|role| role.get_role_permissions().contains(&permission))
        }
    }

    pub async fn get_by_id(
        user: Uuid,
        tournament: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<TournamentUser, OmniError> {
        let user = User::get_by_id(user, pool).await?;
        let roles = user.get_roles(tournament, pool).await?;
        Ok(TournamentUser { user, roles })
    }

    pub async fn get_by_handle(
        handle: &str,
        tournament: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<TournamentUser, OmniError> {
        let user = User::get_by_handle(handle, pool).await?;
        let roles = user.get_roles(tournament, pool).await?;
        Ok(TournamentUser { user, roles })
    }
}

#[test]
fn construct_tournament_user() {
    let org = TournamentUser {
        user: User {
            id: Uuid::now_v7(),
            handle: String::from("some_org"),
            picture_link: Some(PhotoUrl::new("https://i.imgur.com/hbrb2U0.png").unwrap()),
        },
        roles: vec![Role::Organizer, Role::Judge, Role::Marshall],
    };
    assert!(org.has_permission(Permission::DeleteUsers));
}
