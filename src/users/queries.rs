use super::{photourl::PhotoUrl, roles::Role, TournamentUser, User};
use crate::omni_error::OmniError;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
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
    pub async fn get_all(pool: &Pool<Postgres>) -> Result<Vec<User>, OmniError> {
        let users = sqlx::query!("SELECT id, handle, picture_link FROM users")
            .fetch_all(pool)
            .await?
            .iter()
            .map(|u| {
                Ok(User {
                    id: u.id,
                    handle: u.handle.clone(),
                    profile_picture: match u.picture_link.clone() {
                        Some(url) => Some(PhotoUrl::new(&url)?),
                        None => None,
                    },
                })
            })
            .collect::<Result<Vec<User>, OmniError>>()?;
        Ok(users)
    }
    pub async fn create(
        user: User,
        pass: String,
        pool: &Pool<Postgres>,
    ) -> Result<User, OmniError> {
        let pic = match &user.profile_picture {
            Some(url) => Some(url.as_url().to_string()),
            None => None,
        };
        let hash = {
            let argon = Argon2::default();
            let salt = SaltString::generate(&mut OsRng);
            match argon.hash_password(pass.as_bytes(), &salt) {
                Ok(hash) => hash.to_string(),
                Err(e) => return Err(e)?,
            }
        };
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
    // ---------- DATABASE HELPERS ----------
    pub async fn get_roles(
        &self,
        tournament: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Role>, OmniError> {
        let roles_result = sqlx::query!(
            "SELECT roles FROM roles WHERE user_id = $1 AND tournament_id = $2",
            self.id,
            tournament
        )
        .fetch_optional(pool)
        .await?;

        if roles_result.is_none() {
            return Ok(vec![]);
        }

        let roles = roles_result.unwrap().roles;
        let vec = match roles {
            Some(vec) => vec
                .iter()
                .map(|role| serde_json::from_str(role.as_str()))
                .collect::<Result<Vec<Role>, JsonError>>()?,
            None => vec![],
        };

        Ok(vec)
    }

    pub async fn has_role(
        &self,
        role: Role,
        tournament_id: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<bool, OmniError> {
        let roles = self.get_roles(tournament_id, pool).await?;
        return Ok(roles.contains(&role));
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
