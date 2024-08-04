use crate::{parser::Schema, basics::table::Table, file::data::LoadMode};

use super::data::Data;

pub trait DatabaseWriter {
    fn write(&mut self) -> Result<(), String>;
}

impl DatabaseWriter for Schema {
    fn write(&mut self) -> Result<(), String> {
        for table in &mut self.tables {
            table.write()?;
        }

        Ok(())
    }
}

impl DatabaseWriter for Table {
    fn write(&mut self) -> Result<(), String> {
        self.data.write()
    }
}

impl DatabaseWriter for Data {
    fn write(&mut self) -> Result<(), String> {
        match self.load_mode {
            LoadMode::Memory => self.write_memory(),
            LoadMode::Disk => self.write_disk(),
        }
    }
}
