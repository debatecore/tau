use permissions::Permission;
use photourl::PhotoUrl;
use roles::Role;
use uuid::Uuid;

pub mod auth;
pub mod permissions;
pub mod photourl;
mod queries;
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
}

impl TournamentUser {
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.roles
            .iter()
            .any(|role| role.get_role_permissions().contains(&permission))
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
