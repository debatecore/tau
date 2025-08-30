use crate::{
    tournament::roles::Role,
    users::{permissions::Permission as P, UserPatch},
};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand::rngs::OsRng;
use serde::Deserialize;
use serde_json::Error as JsonError;
use sqlx::{query, Pool, Postgres};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    omni_error::OmniError,
    tournament::Tournament,
    users::{photourl::PhotoUrl, TournamentUser, User},
};

impl User {
    pub async fn get_by_id(id: Uuid, pool: &Pool<Postgres>) -> Result<User, OmniError> {
        let user =
            sqlx::query!("SELECT handle, picture_link FROM users WHERE id = $1", id)
                .fetch_one(pool)
                .await?;

        Ok(User {
            id,
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

    pub async fn create(
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
            picture_link,
        };
        self.update_data(&patch, pool).await?;
        Ok(updated_user)
    }

    pub async fn change_password(
        &self,
        new_password: &str,
        pool: &Pool<Postgres>,
    ) -> Result<(), OmniError> {
        let password_hash = User::generate_password_hash(new_password).unwrap().clone();
        match query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            password_hash,
            self.id
        )
        .execute(pool)
        .await
        {
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

    async fn update_data(
        &self,
        patch: &UserPatch,
        pool: &Pool<Postgres>,
    ) -> Result<(), OmniError> {
        let picture_link = match &patch.picture_link {
            Some(url) => Some(url.as_url().to_string()),
            None => Some(self.picture_link.as_ref().unwrap().as_str().to_owned()),
        };
        match query!(
            "UPDATE users SET handle = $1, picture_link = $2 WHERE id = $3",
            patch.handle,
            picture_link,
            self.id
        )
        .execute(pool)
        .await
        {
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

    pub async fn can_create_users_within_any_tournament(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<bool, OmniError> {
        let tournaments = Tournament::get_all(pool).await?;
        for tournament in tournaments {
            let tournament_user =
                TournamentUser::get_by_id(self.id, tournament.id, &pool).await?;
            if tournament_user.has_permission(P::CreateUsersManually)
                || tournament_user.has_permission(P::CreateUsersWithLink)
            {
                return Ok(true);
            }
        }
        return Ok(false);
    }

    /// Invalidates all sessions; implementations must promptly log the user out.
    pub async fn invalidate_all_sessions(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<(), OmniError> {
        match query!("DELETE FROM sessions WHERE user_id = $1", self.id)
            .execute(pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
}
