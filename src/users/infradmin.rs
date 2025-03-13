use super::User;
use crate::{
    omni_error::OmniError,
    users::{permissions::Permission, roles::Role, TournamentUser},
};
use sqlx::{Pool, Postgres};
use strum::VariantArray;
use tracing::{error, info};
use uuid::Uuid;

impl User {
    pub fn is_infrastructure_admin(&self) -> bool {
        self.id.is_max()
    }

    pub fn new_infrastructure_admin() -> Self {
        User {
            id: Uuid::max(),
            handle: String::from("admin"),
            picture_link: None,
        }
    }
}

pub async fn guarantee_infrastructure_admin_exists(pool: &Pool<Postgres>) {
    match sqlx::query!("SELECT * FROM users WHERE id = $1", Uuid::max())
        .fetch_optional(pool)
        .await
    {
        Ok(Some(_)) => (),
        Ok(None) => {
            let admin = User::new_infrastructure_admin();
            match User::post(admin, "admin".to_string(), pool).await {
                Ok(_) => info!("Infrastructure admin created."),
                Err(e) => {
                    let err = OmniError::from(e);
                    error!("Could not create infrastructure admin.");
                    error!("{err}");
                    panic!();
                }
            };
        }
        Err(e) => {
            let err = OmniError::from(e);
            error!("Could not guarantee infrastructure admin's existence.");
            error!("{err}");
            panic!();
        }
    };
}

#[test]
fn construct_infradmin() {
    let infradmin = User::new_infrastructure_admin();
    assert!(infradmin.is_infrastructure_admin());
    let permissions: Vec<Permission> = Permission::VARIANTS.to_vec();
    let roles: Vec<Role> = vec![];
    let user = TournamentUser {
        user: infradmin,
        roles: roles,
    };
    for permission in permissions {
        assert!(user.has_permission(permission))
    }
}
