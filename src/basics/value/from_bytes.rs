use std::{string::FromUtf8Error, array::TryFromSliceError};

use crate::basics::column::{ColumnType, TextType, NumericType, TimestampType};

use super::{Value, NumericValue, TimestampValue};

pub trait FromBytes {
    type EnumType;
    fn from_bytes(bytes: &[u8], enum_type: &Self::EnumType) -> Result<Self, String> where Self: Sized;
}

impl FromBytes for Value {
    type EnumType = ColumnType;

    fn from_bytes(bytes: &[u8], column_type: &ColumnType) -> Result<Self, String> {
        // HINT: temporary solution for NULL values, currently only for Text
        // TODO: implement a different way to store NULL values
        if matches!(column_type, ColumnType::Text(_)) && bytes.iter().all(|b| *b == 0) {
            return Ok(Value::Null)
        }

        let value = match column_type {
            ColumnType::Text(text_type) => {
                let map_err = |e: FromUtf8Error| e.to_string();
                let text = match text_type {
                    TextType::Char => String::from_utf8(bytes.to_vec()).map_err(map_err)?,
                    // TextType::Variable => {
                    //     let s = String::from_utf8(bytes.to_vec()).map_err(map_err)?;
                    //     Value::Text(s)
                    // },
                    TextType::Fixed(_) => {
                        let bytes = bytes.to_vec();
                        String::from_utf8(bytes).map_err(map_err)?
                            .trim_end_matches('\0')
                            .to_string()
                    }
                    
                    _ => todo!()
                };

                Value::Text(text)
            },
            ColumnType::Numeric(numeric_type) => {
                let v = NumericValue::from_bytes(bytes, numeric_type)?;
                Value::Numeric(v)
            },
            ColumnType::Timestamp(timestamp_type) => {
                let v = TimestampValue::from_bytes(bytes, timestamp_type)?;
                Value::Timestamp(v)
            },
            ColumnType::Boolean => Value::Boolean(bytes[0] != 0),

            _ => todo!("Value::from_bytes for {:?}", column_type)
        };

        Ok(value)
    }
}

impl FromBytes for NumericValue {
    type EnumType = NumericType;

    fn from_bytes(bytes: &[u8], numeric_type: &NumericType) -> Result<Self, String> where Self: Sized {
        let map_err = |e: TryFromSliceError| e.to_string(); 

        let numeric_value = match numeric_type {
            NumericType::IntI8 => NumericValue::IntI8(i8::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntI16 => NumericValue::IntI16(i16::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntI32 => NumericValue::IntI32(i32::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntI64 => NumericValue::IntI64(i64::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntU8 => NumericValue::IntU8(u8::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntU16 => NumericValue::IntU16(u16::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntU32 => NumericValue::IntU32(u32::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntU64 => NumericValue::IntU64(u64::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::Float32 => NumericValue::Float32(f32::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::Float64 => NumericValue::Float64(f64::from_be_bytes(bytes.try_into().map_err(map_err)?)),
        };

        Ok(numeric_value)
    }
}

impl FromBytes for TimestampValue {
    type EnumType = TimestampType;

    fn from_bytes(bytes: &[u8], timestamp_type: &Self::EnumType) -> Result<Self, String> where Self: Sized {
        let map_err = |e: TryFromSliceError| e.to_string(); 

        let timestamp_value = match timestamp_type {
            TimestampType::Seconds => TimestampValue::Seconds(u64::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            TimestampType::Milliseconds => TimestampValue::Milliseconds(u64::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            TimestampType::Microseconds => TimestampValue::Microseconds(u64::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            TimestampType::Nanoseconds => TimestampValue::Nanoseconds(u64::from_be_bytes(bytes.try_into().map_err(map_err)?)),
        };

        Ok(timestamp_value)
    }
}
