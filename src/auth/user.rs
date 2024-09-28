use super::{Privilege, Role};

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub hash: String,
    pub roles: Vec<Role>,
    pub privileges: Vec<Privilege>,
    pub is_superuser: bool,
}

impl User {
    pub fn new(name: &str, hash: &str) -> Self {
        Self {
            name: name.to_owned(),
            hash: hash.to_owned(),
            roles: Vec::new(),
            privileges: Vec::new(),
            is_superuser: false,
        }
    }

    pub fn add_role(&mut self, role: Role) {
        self.roles.push(role);
    }
}
