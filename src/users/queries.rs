use super::{photourl::PhotoUrl, TournamentUser, User, UserPatch};
use crate::{
    omni_error::OmniError,
    tournament::{roles::Role, Tournament},
};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use sqlx::{query, Pool, Postgres};
use uuid::Uuid;

impl User {
    pub async fn get_by_id(id: Uuid, pool: &Pool<Postgres>) -> Result<User, OmniError> {
        let user = query!("SELECT handle, picture_link FROM users WHERE id = $1", id)
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
        let user = query!(
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
        let users = query!("SELECT id, handle, picture_link FROM users")
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
        pass: String,
        pool: &Pool<Postgres>,
    ) -> Result<User, OmniError> {
        let pic = match &user.picture_link {
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
        match query!(
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

        let roles = roles_result.unwrap().roles;
        let mut parsed_roles: Vec<Role> = vec![];
        match roles {
            Some(vec) => {
                for value in vec {
                    parsed_roles.push(Role::try_from(value)?);
                }
                return Ok(parsed_roles);
            }
            None => return Ok(vec![]),
        }
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
        if patch.password != None {
            self.update_user_and_change_password(&patch, pool).await?;
        }
        self.update_user_without_changing_password(&patch, pool)
            .await?;
        Ok(updated_user)
    }

    async fn update_user_and_change_password(
        &self,
        patch: &UserPatch,
        pool: &Pool<Postgres>,
    ) -> Result<(), OmniError> {
        let picture_link = match &patch.picture_link {
            Some(url) => Some(url.as_url().to_string()),
            None => Some(self.picture_link.as_ref().unwrap().as_str().to_owned()),
        };
        let password_hash =
            User::generate_password_hash(&patch.password.as_ref().unwrap())
                .unwrap()
                .clone();
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

    async fn update_user_without_changing_password(
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

    pub async fn is_organizer_of_any_tournament(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<bool, OmniError> {
        let tournaments = Tournament::get_all(pool).await?;
        for tournament in tournaments {
            let roles = self.get_roles(tournament.id, pool).await?;
            if roles.contains(&Role::Organizer) {
                return Ok(true);
            }
        }
        return Ok(false);
    }

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
