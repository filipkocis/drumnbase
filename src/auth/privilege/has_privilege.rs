use crate::auth::{User, Role};

use super::{Privilege};

pub trait HasPrivilege {
    fn has_privilege(&self, privilege: &Privilege) -> bool;
    fn has_privileges(&self, privilege: &[Privilege]) -> bool {
        privilege.iter().all(|p| self.has_privilege(p))
    }
}

impl HasPrivilege for User {
    fn has_privilege(&self, privilege: &Privilege) -> bool {
        if self.privileges.contains(privilege) {
            return true
        }

        self.roles.iter().any(|role| role.has_privilege(privilege))
    }
}

impl HasPrivilege for Role {
    fn has_privilege(&self, privilege: &Privilege) -> bool {
        self.privileges.contains(privilege)
    }
}
