pub enum TextType {
    Char,
    Variable,
    Fixed(u32),
}

pub enum NumericType {
    IntU8,
    IntU16,
    IntU32,
    IntU64,

    IntI8,
    IntI16,
    IntI32,
    IntI64,

    Float32,
    Float64,
}

pub enum TimestampType {
    Date,
    Time,
    DateTime,
    TimeStamp,
}


pub enum ColumnType {
    Numeric(NumericType),
    Text(TextType),
    Timestamp(TimestampType),
    Boolean,
    Binary,
    Array,
    Enum,
    UUID,
}

pub struct Column {
    pub name: String,
    pub data_type: ColumnType,
    pub length: u32,
    pub default: String,
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
            default: String::new(),
            not_null: false,
            unique: false,
            read_only: false,
        }
    }
}
