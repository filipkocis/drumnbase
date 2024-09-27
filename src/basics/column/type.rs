#[derive(Debug)]
pub enum TextType {
    Char,
    Variable,
    Fixed(u32),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum TimestampType {
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
    // Date,
    // Time,
}

#[derive(Debug)]
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
