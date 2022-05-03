use std::collections::HashMap;
/// Processor is a main struct for csv editing.
///
/// * Usage
/// ```rust
/// use ced::Processor;
/// let mut processor = Processor::new();
/// processor.import_from_file("test.csv", true).unwrap();
///
/// processor.add_row_from_strings(processor.last_row_index(), &vec!["a","b"]).unwrap();
///
/// processor.overwrite_to_file(true).unwrap();
///
/// let data = processor.get_data();
/// ```
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{CedError, CedResult};
use crate::utils;
use crate::value::{Value, ValueLimiter, ValueType};
use crate::virtual_data::{Column, Row, VirtualData};

const ALPHABET: [&str; 26] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z",
];

/// Csv processor
pub struct Processor {
    pub(crate) file: Option<PathBuf>,
    pages: HashMap<String,VirtualData>,
    pub(crate) cursor: Option<String>,
    pub(crate) print_logs: bool,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            file: None,
            pages: HashMap::new(),
            cursor : None,
            print_logs: true,
        }
    }

    #[allow(dead_code)]
    /// Change current page 
    ///
    /// This doesn't affect page itself but change cursor.
    pub(crate) fn change_page(&mut self, page: &str) -> CedResult<()> {
        if !self.pages.contains_key(page) { 
            return Err(CedError::InvalidPageOperation(
                    format!("No such page \"{}\"", page)
            )) 
        } else {
            self.cursor = Some(page.to_owned());
            Ok(())
        }
    }

    /// Add new page
    pub(crate) fn add_page(&mut self, page: &str, data: &str, has_header: bool) -> CedResult<()> {
        if self.pages.contains_key(page) { 
            return Err(CedError::InvalidPageOperation(
                    format!("\"{}\" already exists", page)
            )) 
        } else {
            // TODO
            self.pages.insert(page.to_owned(), VirtualData::new());
            self.cursor = Some(page.to_owned());
            // IMPORTANT
            // This should come later because import is applied to current **CURSOR** page
            self.import_from_string(data, has_header)
        }
    }

    #[allow(dead_code)]
    /// Try getting page data
    ///
    /// This return an option of data's mutable reference
    pub(crate) fn try_get_page_data(&mut self) -> Option<&mut VirtualData> {
        if let Some(cursor) = self.cursor.as_ref() {
            self.pages.get_mut(cursor)
        } else {
            None
        }
    }

    /// Try get page data but panic if cursor is empty
    ///
    /// This method assumes cursor is set and valid.
    ///
    /// This return data's mutable reference as result
    pub(crate) fn get_page_data_mut(&mut self) -> CedResult<&mut VirtualData> {
        self.pages.get_mut(self.cursor.as_ref().unwrap()).ok_or(CedError::InvalidPageOperation(format!("Cannot get page from cursor which is \"{:?}\"", self.cursor)))
    }

    /// Try get page data but panic if cursor is empty
    ///
    /// This method assumes cursor is set and valid.
    ///
    /// This return data's mutable reference as result
    pub(crate) fn get_page_data(&self) -> CedResult<&VirtualData> {
        self.pages.get(self.cursor.as_ref().unwrap()).ok_or(CedError::InvalidPageOperation(format!("Cannot get page from cursor which is \"{:?}\"", self.cursor)))
    }

    pub(crate) fn log(&self, log: &str) -> CedResult<()> {
        if self.print_logs {
            utils::write_to_stdout(log)?;
        }
        Ok(())
    }

    pub fn import_from_string(&mut self, text: impl AsRef<str>, has_header: bool) -> CedResult<()> {
        let mut content = text.as_ref().lines();

        let mut row_count = 1;
        if has_header {
            let header = content.next();
            if let None = header {
                return Err(CedError::InvalidRowData(format!(
                    "Given data does not have a header"
                )));
            }
            self.add_column_array(&header.unwrap().split(',').collect::<Vec<_>>())?;
        }

        let mut row = content.next();
        while let Some(row_src) = row {
            let split = Self::split_csv_row(&row_src);

            // No column data
            if self.get_page_data_mut()?.columns.len() == 0 {
                self.add_column_array(&self.make_arbitrary_column(split.len()))?;
                continue;
            }

            // Given row data has different length with column
            if split.len() != self.get_page_data_mut()?.get_column_count() {
                self.get_page_data_mut()?.drop();
                return Err(CedError::InvalidRowData(format!(
                    "Row of line \"{}\" has different length.",
                    row_count + 1
                )));
            }

            self.add_row_from_strings(self.get_row_count()?, &split)?;
            row = content.next();
            row_count += 1;
        }
        Ok(())
    }

    pub fn import_from_file(&mut self, path: impl AsRef<Path>, has_header: bool) -> CedResult<()> {
        let content = std::fs::read_to_string(&path).map_err(|err| {
            CedError::io_error(
                err,
                &format!("Failed to import file \"{}\"", path.as_ref().display()),
            )
        })?;
        self.add_page(&path.as_ref().display().to_string(), &content, has_header)?;
        self.file.replace(path.as_ref().to_owned());
        Ok(())
    }

    pub fn write_to_file(&self, file: impl AsRef<Path>) -> CedResult<()> {
        let mut file = File::create(file)
            .map_err(|err| CedError::io_error(err, "Failed to open file for write"))?;

        file.write_all(self.get_data_as_text()?.as_bytes())
            .map_err(|err| CedError::io_error(err, "Failed to write csv content to file"))?;
        Ok(())
    }

    /// Overwrite virtual data's content into imported file
    ///
    /// * cache - whether to backup original file's content into temp directory
    pub fn overwrite_to_file(&self, cache: bool) -> CedResult<bool> {
        if let None = self.file {
            return Ok(true);
        }

        let file = self.file.as_ref().unwrap();
        let csv = self.get_data_as_text()?;
        // Cache file into temp directory
        if cache {
            std::fs::copy(file, std::env::temp_dir().join("cache.csv"))
                .map_err(|err| CedError::io_error(err, "Failed to create cache for overwrite"))?;
        }
        std::fs::write(file, csv.as_bytes())
            .map_err(|err| CedError::io_error(err, "Failed to overwrite file with content"))?;
        Ok(false)
    }

    pub fn edit_cell(&mut self, x: usize, y: usize, input: &str) -> CedResult<()> {
        self.get_page_data_mut()?.set_cell_from_string(x, y, input)?;
        Ok(())
    }

    pub fn edit_column(&mut self, column: &str, input: &str) -> CedResult<()> {
        self.get_page_data_mut()?
            .set_column(column, Value::Text(input.to_owned()))?;
        Ok(())
    }

    pub fn edit_row(&mut self, row_number: usize, input: Vec<Option<Value>>) -> CedResult<()> {
        self.get_page_data_mut()?.edit_row(
            row_number,
            input
        )?;
        Ok(())
    }

    pub fn set_row(&mut self, row_number: usize, input: Vec<Value>) -> CedResult<()> {
        self.get_page_data_mut()?.set_row(
            row_number,
            input
        )?;
        Ok(())
    }

    pub fn set_row_from_string(&mut self, row_number: usize, input: &Vec<impl AsRef<str>>) -> CedResult<()> {
        self.get_page_data_mut()?.set_row(
            row_number,
            input.iter()
                .map(|s| Value::Text(s.as_ref().to_owned()))
                .collect(),
        )?;
        Ok(())
    }

    pub fn add_row(&mut self, row_number: usize, values: Option<&Vec<Value>>) -> CedResult<()> {
        self.get_page_data_mut()?.insert_row(row_number, values)?;
        Ok(())
    }

    pub fn add_row_from_strings(
        &mut self,
        row_number: usize,
        src: &Vec<impl AsRef<str>>,
    ) -> CedResult<()> {
        let values = src
            .iter()
            .map(|a| Value::Text(a.as_ref().to_string()))
            .collect::<Vec<Value>>();
        self.add_row(row_number, Some(&values))?;
        Ok(())
    }

    pub fn add_column(
        &mut self,
        column_number: usize,
        column_name: &str,
        column_type: ValueType,
        limiter: Option<ValueLimiter>,
        placeholder: Option<Value>
    ) -> CedResult<()> {
        self.get_page_data_mut()?
            .insert_column(column_number, column_name, column_type, limiter, placeholder)?;
        Ok(())
    }

    pub fn remove_row(&mut self, row_number: usize) -> CedResult<Option<Row>> {
        Ok(self.get_page_data_mut()?.delete_row(row_number))
    }

    pub fn remove_column(&mut self, column_number: usize) -> CedResult<()> {
        self.get_page_data_mut()?.delete_column(column_number)?;
        Ok(())
    }

    pub fn add_column_array(&mut self, columns: &Vec<impl AsRef<str>>) -> CedResult<()> {
        for col in columns {
            let column_count = self.get_page_data_mut()?.get_column_count();
            self.add_column(
                column_count,
                &col.as_ref(),
                ValueType::Text,
                None,
                None
            )?;
        }
        Ok(())
    }

    pub fn move_row(&mut self, src: usize, target: usize) -> CedResult<()> {
        self.get_page_data_mut()?.move_row(src, target)?;
        Ok(())
    }

    pub fn move_column(&mut self, src: usize, target: usize) -> CedResult<()> {
        self.get_page_data_mut()?.move_column(src, target)?;
        Ok(())
    }

    pub fn rename_column(&mut self, column: &str, new_name: &str) -> CedResult<()> {
        self.get_page_data_mut()?.rename_column(column, new_name)?;
        Ok(())
    }

    pub fn export_schema(&self) -> CedResult<String> {
        Ok(self.get_page_data()?.export_schema())
    }

    pub fn set_schema(&mut self, path: impl AsRef<Path>, panic: bool) -> CedResult<()> {
        let content = std::fs::read_to_string(&path).map_err(|err| {
            CedError::io_error(
                err,
                &format!("Failed to import file \"{}\"", path.as_ref().display()),
            )
        })?;
        let mut content = content.lines();

        let header = content.next();
        if let None = header {
            return Err(CedError::InvalidRowData(format!(
                "Given file does not have a header"
            )));
        }

        let mut row = content.next();
        while let Some(row_src) = row {
            let row_args = row_src.split(',').collect::<Vec<&str>>();
            let limiter = ValueLimiter::from_line(&row_args[1..].to_vec())?;
            self.set_limiter(row_args[0], limiter, panic)?;
            row = content.next();
        }
        Ok(())
    }

    pub fn set_limiter(
        &mut self,
        column: &str,
        limiter: ValueLimiter,
        panic: bool,
    ) -> CedResult<()> {
        let column = self
            .get_page_data_mut()?
            .try_get_column_index(column)
            .ok_or(CedError::InvalidColumn(format!(
                "{} is not a valid column",
                column
            )))?;
        self.get_page_data_mut()?.set_limiter(column, limiter, panic)?;
        Ok(())
    }

    // <MISC>
    pub fn get_row_count(&self) -> CedResult<usize> {
        Ok(self.get_page_data()?.get_row_count())
    }

    pub fn get_column_count(&self) -> CedResult<usize> {
        Ok(self.get_page_data()?.get_column_count())
    }

    /// Get last row index
    pub fn last_row_index(&self) -> CedResult<usize> {
        Ok(self.get_page_data()?.get_row_count().max(1) - 1)
    }

    /// Get last column index
    pub fn last_column_index(&self) -> CedResult<usize> {
        Ok(self.get_page_data()?.get_column_count().max(1) - 1)
    }

    /// Get virtual data as string form
    pub fn get_data_as_text(&self) -> CedResult<String> {
        Ok(self.get_page_data()?.to_string())
    }

    pub fn get_cell(&self, row: usize, column: usize) -> CedResult<Option<&Value>> {
        self.get_page_data()?.get_cell(row, column)
    }

    pub fn get_row(&self, index: usize) -> CedResult<Option<&Row>> {
        Ok(self.get_page_data()?.rows.get(index))
    }

    pub fn get_row_mut(&mut self, index: usize) -> CedResult<Option<&mut Row>> {
        Ok(self.get_page_data_mut()?.rows.get_mut(index))
    }

    pub fn get_column_mut(&mut self, index: usize) -> CedResult<Option<&mut Column>> {
        Ok(self.get_page_data_mut()?.columns.get_mut(index))
    }

    pub fn get_column(&self, index: usize) -> CedResult<Option<&Column>> {
        Ok(self.get_page_data()?.columns.get(index))
    }
    // </MISC>

    // <DRY>
    /// This creates arbritrary column name which is unique for computer
    ///
    /// Name starts with alphabetical order and lengthend sequentially
    ///
    /// e.g.)
    /// a,b,c ... aa,bb,cc ... aaa,bbb,ccc
    fn make_arbitrary_column(&self, size: usize) -> Vec<String> {
        let mut column_names: Vec<String> = vec![];
        for index in 0..size {
            let index = index + 1;
            let target = ALPHABET[index % ALPHABET.len() - 1];
            let name = target.repeat(index / ALPHABET.len() + 1);
            column_names.push(name);
        }
        column_names
    }

    // </DRY>
    fn split_csv_row(line: &str) -> Vec<String> {
        let mut split = vec![];
        let mut on_quote = false;
        let mut previous = ' ';
        let mut chunk = String::new();
        for ch in line.chars() {
            match ch {
                '"' => {
                    // Add literal double quote if previous was same character
                    if previous == '"' {
                        previous = ' '; // Reset previous
                    } else {
                        on_quote = !on_quote;
                        previous = ch;
                        continue;                    
                    }
                },
                ',' => {
                    if !on_quote {
                        let flushed = std::mem::replace(&mut chunk, String::new());
                        split.push(flushed);
                        previous = ch;
                        continue;
                    }
                },
                _ => previous = ch,
            }
            chunk.push(ch);
        }
        split.push(chunk);
        split
    }
}
