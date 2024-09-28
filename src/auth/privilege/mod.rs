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
        name: String,
        table: String,
        action: ColumnAction,
    },
    Function {
        name: String,
        action: FunctionAction,
    },
}

impl Privilege {
    pub fn database(name: &str, action: DatabaseAction) -> Self {
        Self::Database { name: name.to_owned(), action }
    }

    pub fn table(name: &str, action: TableAction) -> Self {
        Self::Table { name: name.to_owned(), action }
    }

    pub fn column(name: &str, action: ColumnAction, table: &str) -> Self {
        Self::Column { name: name.to_owned(), table: table.to_owned(), action }
    }
    
    pub fn function(name: &str, action: FunctionAction) -> Self {
        Self::Function { name: name.to_owned(), action }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Database { name, .. } => name,
            Self::Table { name, .. } => name,
            Self::Column { name, .. } => name,
            Self::Function { name, .. } => name,
        }
    }

    pub fn action(&self) -> &str {
        match self {
            Self::Database { action, .. } => action.as_str(),
            Self::Table { action, .. } => action.as_str(),
            Self::Column { action, .. } => action.as_str(),
            Self::Function { action, .. } => action.as_str(),
        }
    }

    pub fn column_table(&self) -> Option<&str> {
        match self {
            Self::Column { table, .. } => Some(table),
            _ => None,
        }
    }
}

impl Privilege {
    pub fn from_fields(object: &str, object_name: &str, action: &str, table: Option<&str>) -> Result<Self, String> {
        match (object, table) {
            ("column", None) => return Err("missing field table for column privilege".to_owned()),
            ("column", Some(_)) => (),
            (_, Some(_)) => return Err("unexpected field table for non-column privilege".to_owned()),
            _ => (),
        }

        let privilege = match object {
            "database" => Self::database(object_name, DatabaseAction::from_str(action)?),
            "table" => Self::table(object_name, TableAction::from_str(action)?),
            "column" => Self::column(object_name, ColumnAction::from_str(action)?, table.unwrap()),
            "function" => Self::function(object_name, FunctionAction::from_str(action)?),
            _ => return Err(format!("invalid privilege object '{}'", object))
        };

        Ok(privilege)
    }
}
