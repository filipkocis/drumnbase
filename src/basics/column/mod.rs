mod validate;
mod r#type;
mod parse;

pub use r#type::*;

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub data_type: ColumnType,
    pub length: u32,
    pub default: Option<String>,
    pub not_null: bool,
    pub unique: bool,
    pub read_only: bool,
    // pub primary_key: bool,
    // pub foreigh_key: bool,
    // pub check: bool,
    // pub references: String,
    // pub check_constraint: String,
    // pub privileges: Vec<Privilege>,
}

impl Column {
    pub fn new(name: &str, data_type: ColumnType) -> Self {
        Column {
            name: name.to_owned(),
            data_type,
            length: 0,
            default: None,
            not_null: false,
            unique: false,
            read_only: false,
        }
    }

    pub fn set_length(&mut self, length: u32) {
        self.length = length;
    }
}
