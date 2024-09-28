#[derive(Clone)]
pub enum DatabaseAction {
    Create,
    Drop,
    Connect,
    Grant,
}

#[derive(Clone)]
pub enum TableAction {
    Select,
    Insert,
    Update,
    Delete,
    Alter,
    Drop,
    Grant,
}

#[derive(Clone)]
pub enum ColumnAction {
    Update,
    Grant,
}

#[derive(Clone)]
pub enum FunctionAction {
    Execute,
    Grant,
}

impl DatabaseAction {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Create => "create",
            Self::Drop => "drop",
            Self::Connect => "connect",
            Self::Grant => "grant",
        }
    }

    pub fn from_str(action: &str) -> Result<Self, String> {
        let action = match action {
            "create" => Self::Create,
            "drop" => Self::Drop,
            "connect" => Self::Connect,
            "grant" => Self::Grant,
            _ => return Err(format!("invalid database action '{}'", action))
        };

        Ok(action)
    }
}

impl TableAction {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Select => "select",
            Self::Insert => "insert",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Alter => "alter",
            Self::Drop => "drop",
            Self::Grant => "grant",
        }
    }

    pub fn from_str(action: &str) -> Result<Self, String> {
        let action = match action {
            "select" => Self::Select,
            "insert" => Self::Insert,
            "update" => Self::Update,
            "delete" => Self::Delete,
            "alter" => Self::Alter,
            "drop" => Self::Drop,
            "grant" => Self::Grant,
            _ => return Err(format!("invalid table action '{}'", action))
        };

        Ok(action)
    }
}

impl ColumnAction {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Update => "update",
            Self::Grant => "grant",
        }
    }
    
    pub fn from_str(action: &str) -> Result<Self, String> {
        let action = match action {
            "update" => Self::Update,
            "grant" => Self::Grant,
            _ => return Err(format!("invalid column action '{}'", action))
        };

        Ok(action)
    }
}

impl FunctionAction {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Execute => "execute",
            Self::Grant => "grant",
        }
    }

    pub fn from_str(action: &str) -> Result<Self, String> {
        let action = match action {
            "execute" => Self::Execute,
            "grant" => Self::Grant,
            _ => return Err(format!("invalid function action '{}'", action))
        };

        Ok(action)
    }
}
