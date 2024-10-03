use crate::{database::Database, basics::{Table, Column}, function::Function};

use super::{User, PrivilegeFromObject, HasPrivilege};

pub trait Authorize {
    /// Authorize an user to perform an action on the object
    fn authorize(&self, user: &User, action: <Self as PrivilegeFromObject>::Action) -> Result<(), String>
        where Self: PrivilegeFromObject
    {
        if user.is_superuser {
            return Ok(())
        }

        let privilege = self.privilege_for(action);
        let authorized = user.has_privilege(&privilege);

        if authorized {
            Ok(())
        } else {
            Err("unauthorized".to_owned())
        }
    }

    /// Authorize an user to perform multiple actions on the object
    fn authorize_all(&self, user: &User, actions: &[<Self as PrivilegeFromObject>::Action]) -> Result<(), String>
        where Self: PrivilegeFromObject
    {
        if user.is_superuser {
            return Ok(())
        }

        let privileges: Vec<_> = actions.iter().map(|action| self.privilege_for(action.clone())).collect();
        let authorized = user.has_privileges(&privileges);

        if authorized {
            Ok(())
        } else {
            Err("unauthorized".to_owned())
        }
    }
}

impl Authorize for Database {}
impl Authorize for Table {}
impl Authorize for Column {}
impl Authorize for Function {}
