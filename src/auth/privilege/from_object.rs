use crate::{database::Database, basics::Table, function::Function};

use super::{Privilege, action::{DatabaseAction, TableAction, FunctionAction}};

pub trait PrivilegeFromObject {
    /// The type of action that can be performed on the object
    type Action: Clone;

    /// Returns the privilege for the given action on the object
    fn privilege_for(&self, action: Self::Action) -> Privilege;
}

impl PrivilegeFromObject for Database {
    type Action = DatabaseAction;

    fn privilege_for(&self, action: Self::Action) -> Privilege {
        Privilege::database(&self.name, action)
    }
}

impl PrivilegeFromObject for Table {
    type Action = TableAction;

    fn privilege_for(&self, action: Self::Action) -> Privilege {
        Privilege::table(&self.name, action)
    }
}

// HINT: can't implement for column because it needs table name
// impl PrivilegeFromObject for Column {
//     type Action = ColumnAction;
//
//     fn privilege_for(&self, action: Self::Action) -> Privilege {
//         Privilege::column(&self.name, action, &self.table.name
//     }
// }

impl PrivilegeFromObject for Function {
    type Action = FunctionAction;

    fn privilege_for(&self, action: Self::Action) -> Privilege {
        Privilege::function(&self.name, action)
    }
}
