use self::action::{DatabaseAction, TableAction, ColumnAction, FunctionAction};

pub mod action;

pub enum Privilege {
    Database {
        name: String,
        action: DatabaseAction,
    },
    Table {
        name: String,
        action: TableAction,
    },
    Column {
        table: String,
        column: String,
        action: ColumnAction,
    },
    Function {
        name: String,
        action: FunctionAction,
    },
}
