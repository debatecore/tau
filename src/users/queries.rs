use super::{photourl::PhotoUrl, roles::Role, TournamentUser, User};
use crate::omni_error::OmniError;
use serde_json::Error as JsonError;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

impl User {
    pub async fn get_by_id(id: Uuid, pool: &Pool<Postgres>) -> Result<User, OmniError> {
        let user =
            sqlx::query!("SELECT handle, picture_link FROM users WHERE id = $1", id)
                .fetch_one(pool)
                .await?;

        Ok(User {
            id,
            handle: user.handle,
            profile_picture: match user.picture_link {
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
            profile_picture: match user.picture_link {
                Some(url) => Some(PhotoUrl::new(&url)?),
                None => None,
            },
        })
    }
    // ---------- DATABASE HELPERS ----------
    pub async fn get_roles(
        &self,
        tournament: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Role>, OmniError> {
        let roles = sqlx::query!(
            "SELECT roles FROM roles WHERE user_id = $1 AND tournament_id = $2",
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
