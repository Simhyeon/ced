use std::path::{Path, PathBuf};

use crate::value::{ValueType, ValueLimiter, Value};
use crate::virtual_data::{VirtualData, Row};
use crate::models::Direction;
use crate::error::{CedResult, CedError};

pub struct Processor {
    pub(crate)  file  : Option<PathBuf>,
    pub(crate)  cursor: Cursor,
    pub(crate)  data  : VirtualData,
}

// TODO
// Possibly add row, column based edit function such as
// edit_row or edit_column
impl Processor {
    pub fn new() -> Self {
        Self {
            file: None,
            cursor: Cursor { x: 0, y: 0 },
            data: VirtualData::new(),
        }
    }

    pub fn read_from_file(&mut self, path: &Path, has_header: bool) -> CedResult<()> {
        let content = std::fs::read_to_string(path).map_err(|err| CedError::io_error(err, &format!("Failed to read file \"{}\"", path.display())))?;
        let mut content = content.lines();

        if has_header {
            let header = content.next();
            if let None = header {
                return Err(CedError::InvalidRowData(format!("Gien file does not have a header")));
            }
            self.create_columns_fast(&header.unwrap().split(',').collect::<Vec<_>>());
        }
        let mut row = content.next();
        while row != None {
            self.add_row_from_vector(&row.unwrap().split(',').collect::<Vec<&str>>())?;
            row = content.next();
        }
        self.file.replace(path.to_owned());
        Ok(())
    }

    pub fn overwrite(&self) -> CedResult<()> {
        if let None = self.file { return Ok(()); } 

        let file = self.file.as_ref().unwrap();
        let csv = self.data.to_string();
        std::fs::copy(file, std::env::temp_dir().join("cache.csv")).map_err(|err| CedError::io_error(err,"Failed to create cache for overwrite"))?;
        std::fs::write(file,csv.as_bytes()).map_err(|err| CedError::io_error(err, "Failed to overwrite file with content"))?;
        Ok(())
    }

    pub fn edit_cell(&mut self, x: usize, y:usize, input: &str) -> CedResult<()> {
        self.data.set_data_from_raw(x, y, input)?;
        Ok(())
    }

    // TODO
    pub fn edit_row(&mut self) {

    }

    pub fn move_cursor(&mut self, direction: Direction) {
        self.cursor.move_by_direction(direction);
    }

    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.cursor.set(x, y);
    }

    pub fn add_row(&mut self, row_number: usize, values: Option<&Vec<Value>>) -> CedResult<()> {
        self.data.insert_row(row_number, values)?;
        Ok(())
    }

    pub fn remove_row(&mut self, row_number: usize) -> Option<Row> {
        self.data.delete_row(row_number)
    }

    pub fn add_column(&mut self, column_number: usize, column_name: &str,column_type: ValueType, limiter: Option<ValueLimiter>) {
        self.data.insert_column(column_number, column_name, column_type, limiter);
    }

    pub fn remove_column(&mut self, column_number: usize) -> CedResult<()> {
        self.data.delete_column(column_number)?;
        Ok(())
    }

    // Some fast methods
    pub fn add_row_from_vector(&mut self, src: &Vec<impl AsRef<str>>) -> CedResult<()> {
        let values = src.iter().map(|a| Value::Text(a.as_ref().to_string())).collect::<Vec<Value>>();
        self.add_row(self.data.get_row_count(), Some(&values))?;
        Ok(())
    }

    // TODO
    pub fn edit_row_from_vector(&mut self, src: &Vec<impl AsRef<str>>) -> CedResult<()> {
        // let values = src.iter().map(|a| Value::Text(a.as_ref().to_string())).collect::<Vec<Value>>();
        // self.edit_row(self.data.get_row_count(), Some(&values))?;
        Ok(())
    }

    pub fn create_columns_fast(&mut self, columns: &Vec<impl AsRef<str>>) {
        for col in columns {
            self.add_column(self.data.get_column_count(), &col.as_ref(), ValueType::Text, None);
        }
    }
}

pub struct Cursor {
    x: usize,
    y: usize,
}

impl Cursor {
    pub fn set(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    pub fn move_by_direction(&mut self, direction: Direction) {
        let (x,y) = direction.to_vector();
        // Minimal value as 0
        self.x += (self.x as isize + x).min(0) as usize;
        self.y += (self.y as isize + y).min(0) as usize;
    }
}
