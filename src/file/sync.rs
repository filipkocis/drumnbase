use std::io::SeekFrom;

use crate::basics::{table::Table, row::ToBytes};

use super::data::Data;

impl Table {
    pub fn get_row_prefix_length(&self) -> usize {
        // TODO: in the future, this will include bits for the row's metadata
        0     
    }

    pub fn get_row_length(&self) -> usize {
        self.columns
            .iter()
            .fold(
                self.get_row_prefix_length(), 
                |acc, column| acc + column.length as usize
            )
    }

    pub fn get_column_offset(&self, column_index: usize) -> Result<usize, String> {
        if column_index >= self.columns.len() {
            return Err(format!("Column index out of bounds: {}", column_index))
        }

        let mut offset = self.get_row_prefix_length();
        for i in 0..column_index {
            offset += self.columns[i].length as usize;
        }

        Ok(offset)
    }

    pub fn get_row_offset(&self, row_index: usize) -> Result<usize, String> {
        if row_index >= self.data.len() {
            return Err(format!("Row index out of bounds: {}", row_index))
        }

        Ok(row_index * self.get_row_length())
    }
}

impl Table {
    /// Syncs the buffer with the disk and memory, leaving it empty
    pub fn sync_buffer(&mut self) -> Result<(), String> {
        self.data.writer_seek_end()?;
        if self.data.buf_rows.len() == 0 { return Ok(()) }

        for index in 0..self.data.buf_rows.len() {
            let row = &self.data.buf_rows[index];
            let row_bytes = row.convert_to_bytes(&self.columns);
            self.data.writer_write(&row_bytes)?; 
        }
       
        self.data.buffer_apply();
        self.data.writer_flush()?;

        Ok(())
    }

    /// Syncs the row at the given index with the disk
    pub fn sync_row(&mut self, index: usize) {
        todo!()
    }

    /// Syncs the row at the given index with the disk, it only syncs the specified columns
    pub fn sync_row_parts(&mut self, row_index: usize, column_indexes: &Vec<usize>) -> Result<(), String> {
        let mut column_indexes = column_indexes.clone();
        column_indexes.sort_unstable();
        let row = self.data.get(row_index).unwrap();
        let row_offset = self.get_row_offset(row_index)?;

        let buffers_with_col_idx = column_indexes
            .iter()
            .map(|&column_index| {
                let column_length = self.columns[column_index].length;
                let buffer = row.get(column_index).unwrap().to_bytes(column_length);

                (column_index, buffer)
            })
            .collect::<Vec<_>>();
    
        // TODO
        // columns are sorted, so we can iterate and seek through them in order
        for (column_index, buffer) in buffers_with_col_idx {
            let column_offset = self.get_column_offset(column_index)?;

            self.data.writer_seek((row_offset + column_offset) as u64)?;
            self.data.writer_write(&buffer)?;
            self.data.writer_flush()?;
        }

        Ok(())
    }
}
