use crate::{parser::Schema, basics::table::Table, file::data::LoadMode};

pub trait DatabaseReader {
    fn read(&mut self) -> Result<(), String>;
}

impl DatabaseReader for Schema {
    fn read(&mut self) -> Result<(), String> {
        for table in &mut self.tables {
            table.read()?;
        }

        Ok(())
    }
}

impl DatabaseReader for Table {
    fn read(&mut self) -> Result<(), String> {
        match self.data.load_mode {
            LoadMode::Memory => self.data.read_memory(&self.columns),
            LoadMode::Disk => self.data.read_disk(&self.columns),
        }
    }
}
