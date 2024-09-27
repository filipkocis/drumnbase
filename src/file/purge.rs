use crate::{database::Database, basics::table::Table};

use super::data::LoadMode;

pub trait Purge {
    /// Removes rows marked as deleted (disk and memory)
    fn purge(&mut self) -> Result<(), String>;
}

impl Purge for Database {
    fn purge(&mut self) -> Result<(), String> {
        let tables = &mut self.tables;
        for table in tables {
            table.purge()?;
        }
        Ok(())
    }
}

impl Purge for Table {
    fn purge(&mut self) -> Result<(), String> {
        if self.data.load_mode == LoadMode::Disk {
            todo!("Purge for disk mode")
        }

        // early return if no deleted rows
        if !self.data.iter().any(|r| r.is_deleted()) {
            return Ok(())
        }

        // removes deleted rows from memory
        self.data.purge_deleted_rows();

        // rewrites the data in the file to exclude deleted rows
        self.data.writer_seek(0)?;
        for index in 0..self.data.len() {
            let row = self.data.get(index).unwrap();
            let buffer = row.convert_to_bytes(&self.columns);
            self.data.writer_write(&buffer)?;
        }

        // flush the writer buffer and truncate to new size
        self.data.writer_flush()?;
        self.data.writer_truncate()?;

        Ok(())
    }
}
