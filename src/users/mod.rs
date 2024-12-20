use permissions::Permission;
use photourl::PhotoUrl;
use roles::Role;
use ulid::Ulid;

pub mod permissions;
pub mod photourl;
pub mod roles;

pub struct User {
    pub id: Ulid,
    pub username: String,
    pub displayname: String,
    pub profilepicture: Option<PhotoUrl>,
}

pub struct TournamentUser {
    pub user: User,
    pub roles: Vec<Role>,
}

impl User {
    pub fn is_infrastructure_admin(&self) -> bool {
        self.id.is_nil()
    }
    pub fn new_infrastructure_admin() -> Self {
        User {
            id: Ulid::nil(),
            username: String::from("admin"),
            displayname: String::from("admin"),
            profilepicture: None,
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
            id: Ulid::new(),
            username: String::from("org"),
            displayname: String::from("organizator"),
            profilepicture: Some(PhotoUrl::new("https://i.imgur.com/hbrb2U0.png").unwrap()),
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
