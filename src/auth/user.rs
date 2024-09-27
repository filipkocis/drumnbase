use super::{Privilege, Role};

pub struct User {
    pub name: String,
    pub roles: Vec<Role>,
    pub privileges: Vec<Privilege>,
    pub is_superuser: bool,
}
