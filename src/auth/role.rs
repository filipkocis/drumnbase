use super::Privilege;

#[derive(Clone)]
pub struct Role {
    pub name: String,
    pub privileges: Vec<Privilege>,
}

impl Role {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            privileges: Vec::new()
        }
    }

    pub fn add_privilege(&mut self, privilege: Privilege) {
        self.privileges.push(privilege);
    }
}
