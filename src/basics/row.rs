use std::fmt::Display;

use super::{Column, Value, value::{ToBytes, FromBytes}};

impl Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.values.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(" | ");
        write!(f, "{}", string)
    }
}

pub const NULL_BYTE: u8 = 0;
const EMPTY_FLAGS: u8 = 0;
const DELETED_FLAG: u8 = 1;
// const UNUSED_FLAG_2: u8 = 2;
// const UNUSED_FLAG_3: u8 = 4;
// const UNUSED_FLAG_4: u8 = 8;
// const UNUSED_FLAG_5: u8 = 16;
// const UNUSED_FLAG_6: u8 = 32;
// const UNUSED_FLAG_7: u8 = 64;
// const UNUSED_FLAG_8: u8 = 128;

#[derive(Debug, Clone)]
pub struct Row {
    values: Vec<Value>,
    flags: u8,
}

impl Row {
    pub fn new() -> Row {
        Row {
            values: Vec::new(),
            flags: EMPTY_FLAGS,
        }
    }

    pub fn from_values(values: Vec<Value>) -> Row {
        Row {
            values,
            flags: EMPTY_FLAGS,
        }
    }

    pub fn with_flags(flags: u8) -> Row {
        let mut row = Row::new();
        row.flags = flags;
        row
    }

    pub fn get_flags(&self) -> u8 {
        self.flags
    }

    pub fn is_deleted(&self) -> bool {
        self.flags & DELETED_FLAG != 0
    }

    pub fn mark_deleted(&mut self) {
        self.flags |= DELETED_FLAG;
    }

    pub fn add(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    pub fn set(&mut self, index: usize, value: Value) {
        if index >= self.values.len() {
            self.values.resize(index + 1, Value::Null);
        }

        self.values[index] = value;
    }

    pub fn update_with(&mut self, new_values: &Vec<(usize, Value)>) {
        for (column_index, value) in new_values {
            self.set(*column_index, value.clone());
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<Value> {
        if index >= self.values.len() { return None; }
        Some(self.values.remove(index))
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.values.get_mut(index)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<Value> {
        self.values.iter()
    }

    pub fn convert_to_bytes(&self, columns: &Vec<Column>) -> Vec<u8> {
        let mut bytes = vec![self.flags];

        let values = self.values.iter().enumerate().flat_map(|(i, v)| {
            let length = columns[i].length;
            v.to_bytes(length)
        });

        bytes.extend(values);

        bytes
    }

    pub fn convert_from_bytes(bytes: &[u8], columns: &Vec<Column>) -> Result<Self, String> {
        let mut row = Row::new();
        let mut offset = 0;

        // convert prefix
        row.flags = bytes[offset];
        offset += 1;

        // convert values
        for column in columns {
            let length = column.length as usize;
            let value = Value::from_bytes(&bytes[offset..offset + length], &column.data_type)?;
            row.add(value);
            offset += length;
        }
        
        Ok(row)
    }

    /// Returns a new row without the columns at the given indexes.
    pub fn with_excluded_columns(&self, indexes: &[usize]) -> Row {
        let mut row = Row::with_flags(self.flags);
        self.values.iter().enumerate().for_each(|(i, v)| {
            if !indexes.contains(&i) {
                row.add(v.clone());
            }
        });
        row
    }

    /// Returns a new row with only the columns at the given indexes.
    /// Panics if any of the indexes are out of bounds
    pub fn with_kept_columns(&self, indexes: &[usize]) -> Row {
        let mut row = Row::with_flags(self.flags);
        indexes.iter().for_each(|&i| {
            row.add(self.values[i].clone());
        });
        row
    }
}
