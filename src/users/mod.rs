use crate::omni_error::OmniError;
use permissions::Permission;
use photourl::PhotoUrl;
use roles::Role;
use serde_json::Error as JsonError;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub mod permissions;
pub mod photourl;
pub mod roles;

pub struct User {
    pub id: Uuid,
    pub handle: String,
    pub profile_picture: Option<PhotoUrl>,
}

pub struct TournamentUser {
    pub user: User,
    pub roles: Vec<Role>,
}

impl User {
    pub fn is_infrastructure_admin(&self) -> bool {
        self.id.is_max()
    }
    pub fn new_infrastructure_admin() -> Self {
        User {
            id: Uuid::max(),
            handle: String::from("admin"),
            profile_picture: None,
        }
    }
    // ---------- DATABASE ----------
    pub async fn get_by_id(id: Uuid, pool: &Pool<Postgres>) -> Result<User, OmniError> {
        let user =
            sqlx::query!("SELECT handle, pictureLink FROM users WHERE id = $1", id)
                .fetch_one(pool)
                .await?;

        Ok(User {
            id,
            handle: user.handle,
            profile_picture: match user.picturelink {
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
            "SELECT id, pictureLink FROM users WHERE handle = $1",
            handle
        )
        .fetch_one(pool)
        .await?;

        Ok(User {
            id: user.id,
            handle: handle.to_string(),
            profile_picture: match user.picturelink {
                Some(url) => Some(PhotoUrl::new(&url)?),
                None => None,
            },
        })
    }
    // ---------- DATABASE HELPERS ----------
    async fn get_roles(
        &self,
        tournament: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Role>, OmniError> {
        let roles = sqlx::query!(
            "SELECT roles FROM roles WHERE userId = $1 AND tournamentId = $2",
            self.id,
            tournament
        )
        .fetch_one(pool)
        .await?
        .roles;

        let vec = match roles {
            Some(vec) => vec
                .iter()
                .map(|role| serde_json::from_str(role.as_str()))
                .collect::<Result<Vec<Role>, JsonError>>()?,
            None => vec![],
        };

        Ok(vec)
    }
}

impl TournamentUser {
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.roles
            .iter()
            .any(|role| role.get_role_permissions().contains(&permission))
    }
    // ---------- DATABASE ----------
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
            profile_picture: Some(
                PhotoUrl::new("https://i.imgur.com/hbrb2U0.png").unwrap(),
            ),
        },
        roles: vec![Role::Organizer, Role::Judge, Role::Marshall],
    };
    assert!(org.has_permission(Permission::DeleteUsers));
}

#[test]
fn construct_infradmin() {
    let infradmin = User::new_infrastructure_admin();
    assert!(infradmin.is_infrastructure_admin());
}
