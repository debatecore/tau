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
    pub profilepicture: PhotoUrl,
    pub roles: Vec<Role>,
}

impl User {
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.roles
            .iter()
            .any(|role| role.get_role_permissions().contains(&permission))
    }
}

#[test]
fn construct_user() {
    let admin = User {
        id: Ulid::new(),
        username: String::from("admin"),
        displayname: String::from("admin"),
        profilepicture: PhotoUrl::new("https://i.imgur.com/hbrb2U0.png").unwrap(),
        roles: vec![Role::Admin, Role::Organizer, Role::Judge, Role::Marshall],
    };
    admin.has_permission(Permission::DeleteUsers);
}
