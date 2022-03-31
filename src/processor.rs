use std::path::{Path, PathBuf};

use crate::value::{ValueType, ValueLimiter, Value};
use crate::virtual_data::{VirtualData, Row};
use crate::error::{CedResult, CedError};

const ALPHABET: [&str; 26] = ["a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z"];

pub struct Processor {
    pub(crate)  file  : Option<PathBuf>,
    pub(crate)  data  : VirtualData,
}

// TODO
// Possibly add row, column based edit function such as
// edit_row or edit_column
impl Processor {
    pub fn new() -> Self {
        Self {
            file: None,
            data: VirtualData::new(),
        }
    }

    pub fn import_from_file(&mut self, path: &Path, has_header: bool) -> CedResult<()> {
        let content = std::fs::read_to_string(path).map_err(|err| CedError::io_error(err, &format!("Failed to import file \"{}\"", path.display())))?;
        let mut content = content.lines();

        let mut row_count = 1;
        if has_header {
            let header = content.next();
            if let None = header {
                return Err(CedError::InvalidRowData(format!("Given file does not have a header")));
            }
            self.add_column_array(&header.unwrap().split(',').collect::<Vec<_>>());
        } 

        let mut row = content.next();
        while let Some(row_src) = row {
            let split = &row_src.split(',').collect::<Vec<&str>>();

            // No column data
            if self.data.columns.len() == 0 {
                self.add_column_array(&self.make_arbitrary_column(split.len()));
                continue;
            }

            // Given row data has different length with column
            if split.len() != self.data.get_column_count() {
                self.data.drop();
                return Err(CedError::InvalidRowData(format!("Row of line \"{}\" has different length.", row_count + 1)));
            }

            self.add_row_from_strings(self.data.get_row_count(),split)?;
            row = content.next();
            row_count += 1;
        }
        self.file.replace(path.to_owned());
        Ok(())
    }

    pub fn overwrite_to_file(&self) -> CedResult<()> {
        if let None = self.file { return Ok(()); } 

        let file = self.file.as_ref().unwrap();
        let csv = self.data.to_string();
        std::fs::copy(file, std::env::temp_dir().join("cache.csv")).map_err(|err| CedError::io_error(err,"Failed to create cache for overwrite"))?;
        std::fs::write(file,csv.as_bytes()).map_err(|err| CedError::io_error(err, "Failed to overwrite file with content"))?;
        Ok(())
    }

    pub fn edit_cell(&mut self, x: usize, y:usize, input: &str) -> CedResult<()> {
        self.data.set_data_from_string(x, y, input)?;
        Ok(())
    }

    pub fn edit_column(&mut self, column : &str, input: &str) -> CedResult<()> {
        self.data.set_column(column,Value::Text(input.to_owned()))?;
        Ok(())
    }

    pub fn edit_row(&mut self, row_number: usize, input: &str) -> CedResult<()> {
        self.data.set_row(row_number, input.split(",").map(|s| Value::Text(s.to_owned())).collect())?;
        Ok(())
    }

    pub fn add_row(&mut self, row_number: usize, values: Option<&Vec<Value>>) -> CedResult<()> {
        self.data.insert_row(row_number, values)?;
        Ok(())
    }

    pub fn add_row_from_strings(&mut self, row_number: usize,src: &Vec<impl AsRef<str>>) -> CedResult<()> {
        let values = src.iter().map(|a| Value::Text(a.as_ref().to_string())).collect::<Vec<Value>>();
        self.add_row(row_number, Some(&values))?;
        Ok(())
    }

    pub fn add_column(&mut self, column_number: usize, column_name: &str,column_type: ValueType, limiter: Option<ValueLimiter>) {
        self.data.insert_column(column_number, column_name, column_type, limiter);
    }

    pub fn remove_row(&mut self, row_number: usize) -> Option<Row> {
        self.data.delete_row(row_number)
    }

    pub fn remove_column(&mut self, column_number: usize) -> CedResult<()> {
        self.data.delete_column(column_number)?;
        Ok(())
    }

    pub fn add_column_array(&mut self, columns: &Vec<impl AsRef<str>>) {
        for col in columns {
            self.add_column(self.data.get_column_count(), &col.as_ref(), ValueType::Text, None);
        }
    }

    pub fn move_row(&mut self, src: usize, target: usize) -> CedResult<()> {
        self.data.move_row(src,target)?;
        Ok(())
    }

    pub fn move_column(&mut self, src: usize, target: usize) -> CedResult<()> {
        self.data.move_column(src,target)?;
        Ok(())
    }

    pub fn rename_column(&mut self, column : &str, new_name: &str) -> CedResult<()> {
        self.data.rename_column(column, new_name)?;
        Ok(())
    }

    // <DRY>
    fn make_arbitrary_column(&self, size: usize) -> Vec<String> {
        let mut column_names : Vec<String> = vec![];
        for index in 0..size {
            let index = index + 1;
            let target = ALPHABET[index % ALPHABET.len() - 1];
            let name = target.repeat(index / ALPHABET.len()  + 1);
            column_names.push(name);
        }
        column_names
    }

    // </DRY>
}
