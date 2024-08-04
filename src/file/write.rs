use crate::{parser::Schema, basics::table::Table, file::data::LoadMode};

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
        match self.data.load_mode {
            LoadMode::Memory => self.data.write_memory(&self.columns),
            LoadMode::Disk => self.data.write_disk(&self.columns),
        }
    }
}
