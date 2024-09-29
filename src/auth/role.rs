use std::collections::HashSet;

use super::Privilege;

#[derive(Clone, Debug)]
pub struct Role {
    pub name: String,
    pub privileges: HashSet<Privilege>,
}

impl Role {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            privileges: HashSet::new()
        }
    }

    /// Add a privilege to the role, returns true if it's a new privilege
    pub fn add_privilege(&mut self, privilege: Privilege) -> bool {
        self.privileges.insert(privilege)
    }
}
