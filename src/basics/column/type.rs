#[derive(Debug, Clone)]
pub enum TextType {
    Char,
    Variable,
    Fixed(u32),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum TimestampType {
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
    // Date,
    // Time,
}

#[derive(Debug, Clone)]
pub enum ColumnType {
    Numeric(NumericType),
    Text(TextType),
    Timestamp(TimestampType),
    Boolean,
    Binary,
    Array(Box<ColumnType>),
    Enum,
    UUID,
}

impl ColumnType {
    pub fn len(&self) -> u32 {
        match self {
            ColumnType::Numeric(n) => match n {
                NumericType::IntU8 => 1,
                NumericType::IntU16 => 2,
                NumericType::IntU32 => 4,
                NumericType::IntU64 => 8,

                NumericType::IntI8 => 1,
                NumericType::IntI16 => 2,
                NumericType::IntI32 => 4,
                NumericType::IntI64 => 8,

                NumericType::Float32 => 4,
                NumericType::Float64 => 8,
            },
            ColumnType::Text(t) => match t {
                TextType::Char => 1,
                TextType::Variable => todo!("variable length text"),
                TextType::Fixed(len) => *len,
            },
            ColumnType::Timestamp(_) => 8,
            ColumnType::Boolean => 1,
            _ => todo!("column type len for {:?}", self),
        }
    }
}
