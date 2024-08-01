pub enum NumericValue {
    IntU8(u8),
    IntU16(u16),
    IntU32(u32),
    IntU64(u64),

    IntI8(i8),
    IntI16(i16),
    IntI32(i32),
    IntI64(i64),

    Float32(f32),
    Float64(f64),

}

pub enum Value {
    Text(String),
    Numeric(NumericValue),
    Timestamp(i64),
    Boolean(bool),
    Binary(Vec<u8>),
    Array(Vec<Value>),
    Enum(String),
    UUID(String),
}

pub struct Row {
    pub values: Vec<Value>,
}
