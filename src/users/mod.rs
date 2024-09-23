use permissions::Permission;
use roles::Role;
use ulid::Ulid;

pub mod permissions;
pub mod roles;

pub struct User {
    pub id: Ulid,
    pub username: String,
    pub displayname: String,
    // pub profilepicture: PhotoUrl
    pub roles: Vec<Role>,
}

impl User {
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.roles
            .iter()
            .any(|role| role.get_role_permissions().contains(&permission))
    }
}
