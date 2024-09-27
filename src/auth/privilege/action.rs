pub enum DatabaseAction {
    Create,
    Drop,
    Connect,
    Grant,
}

pub enum TableAction {
    Select,
    Insert,
    Update,
    Delete,
    Alter,
    Drop,
    Grant,
}

pub enum ColumnAction {
    Update,
    Grant,
}

pub enum FunctionAction {
    Execute,
    Grant,
}
