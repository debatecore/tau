use axum::http::HeaderMap;
use permissions::Permission;
use photourl::PhotoUrl;
use roles::Role;
use serde::Serialize;
use sqlx::{Pool, Postgres};
use tower_cookies::Cookies;
use tracing::error;
use uuid::Uuid;

use crate::omni_error::OmniError;

pub mod auth;
pub mod infradmin;
pub mod permissions;
pub mod photourl;
pub mod queries;
pub mod roles;

#[derive(Serialize)]
pub struct User {
    pub id: Uuid,
    pub handle: String,
    pub profile_picture: Option<PhotoUrl>,
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
                roles: vec![Role::Admin],
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
}

#[test]
fn construct_tournament_user() {
    let org = TournamentUser {
        user: User {
            id: Uuid::now_v7(),
            handle: String::from("some_org"),
            profile_picture: Some(
                PhotoUrl::new("https://i.imgur.com/hbrb2U0.png").unwrap(),
            ),
        },
        roles: vec![Role::Organizer, Role::Judge, Role::Marshall],
    };
    assert!(org.has_permission(Permission::DeleteUsers));
}
