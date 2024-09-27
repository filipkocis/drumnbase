use super::Privilege;

pub struct Role {
    pub name: String,
    pub privileges: Vec<Privilege>,
}
