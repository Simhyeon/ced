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

#[cfg(feature = "cli")]
use crate::cli::preset::Preset;
use crate::error::{CedError, CedResult};
use crate::page::Page;
use crate::utils;
use dcsv::Column;
use dcsv::{Value, ValueLimiter, ValueType};
use std::collections::HashMap;

/// Csv processor
pub struct Processor {
    pub(crate) file: Option<PathBuf>,
    pub(crate) pages: HashMap<String, Page>,
    pub(crate) cursor: Option<String>,
    pub(crate) print_logs: bool,
    #[cfg(feature = "cli")]
    preset: Preset,
    #[cfg(feature = "cli")]
    pub(crate) no_loop: bool,
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor {
    pub fn new() -> Self {
        Self {
            file: None,
            pages: HashMap::new(),
            cursor: None,
            print_logs: true,
            #[cfg(feature = "cli")]
            preset: Preset::empty(),
            #[cfg(feature = "cli")]
            no_loop: false,
        }
    }

    /// Change current page
    ///
    /// This doesn't affect page itself but change cursor.
    /// * This returns boolean value whether change succeeded
    pub fn change_cursor(&mut self, page_name: &str) -> bool {
        if !self.pages.contains_key(page_name) {
            false
        } else {
            self.cursor = Some(page_name.to_owned());
            true
        }
    }

    /// Get current cursor
    pub fn get_cursor(&self) -> Option<String> {
        self.cursor.as_ref().map(|s| s.to_string())
    }

    /// Add a new page
    pub fn add_page(
        &mut self,
        page: &str,
        data: &str,
        has_header: bool,
        line_ending: Option<char>,
        raw_mode: bool,
    ) -> CedResult<()> {
        if self.pages.contains_key(page) {
            return Err(CedError::InvalidPageOperation(format!(
                "\"{}\" already exists",
                page
            )));
        } else {
            let mut ignore_empty_row = true;
            if let Ok(val) = std::env::var("CED_READ_STRICT") {
                if val.to_lowercase() == "true" {
                    ignore_empty_row = false;
                }
            }
            let mut reader = dcsv::Reader::new()
                .use_line_delimiter(line_ending.unwrap_or('\n'))
                .has_header(has_header)
                .ignore_empty_row(ignore_empty_row);

            let page_data = if raw_mode {
                Page::new_array(reader.array_from_stream(data.as_bytes())?)
            } else {
                Page::new_data(reader.data_from_stream(data.as_bytes())?)
            };
            self.pages.insert(page.to_owned(), page_data);
            self.cursor = Some(page.to_owned());
            Ok(())
        }
    }

    /// Check if processor contains page
    pub fn contains_page(&self, page: &str) -> bool {
        self.pages.contains_key(page)
    }

    /// Try get page data but panic if cursor is empty
    ///
    /// This method assumes cursor is set and valid.
    ///
    /// This return data's mutable reference as result
    pub(crate) fn get_page_data_mut(&mut self, page: &str) -> CedResult<&mut Page> {
        self.pages.get_mut(page).ok_or_else(|| {
            CedError::InvalidPageOperation(format!(
                "Cannot get page from cursor which is \"{:?}\"",
                self.cursor
            ))
        })
    }

    /// Try get page data but panic if cursor is empty
    ///
    /// This method assumes cursor is set and valid but you can set extra argument to return empty
    /// data set.
    pub(crate) fn get_page_data(&self, page: &str) -> CedResult<&Page> {
        self.pages.get(page).ok_or_else(|| {
            CedError::InvalidPageOperation(format!(
                "Cannot get page from cursor which is \"{:?}\"",
                self.cursor
            ))
        })
    }

    pub(crate) fn log(&self, log: &str) -> CedResult<()> {
        if self.print_logs {
            utils::write_to_stdout(log)?;
        }
        Ok(())
    }

    /// Import file content as page
    ///
    /// This will drop the page if given page name already exists.
    pub fn import_from_file(
        &mut self,
        path: impl AsRef<Path>,
        has_header: bool,
        line_ending: Option<char>,
        raw_mode: bool,
    ) -> CedResult<()> {
        let content = std::fs::read_to_string(&path).map_err(|err| {
            CedError::io_error(
                err,
                &format!("Failed to import file \"{}\"", path.as_ref().display()),
            )
        })?;
        let page_name = &path.as_ref().display().to_string();

        // Remove exsiting entry for convenience.
        self.pages.remove_entry(page_name);

        self.add_page(page_name, &content, has_header, line_ending, raw_mode)?;
        self.file.replace(path.as_ref().to_owned());
        Ok(())
    }

    /// Write all page's content into a file
    pub fn write_to_file(&self, page: &str, file: impl AsRef<Path>) -> CedResult<()> {
        let mut file = File::create(file)
            .map_err(|err| CedError::io_error(err, "Failed to open file for write"))?;

        file.write_all(self.get_page_as_string(page)?.as_bytes())
            .map_err(|err| CedError::io_error(err, "Failed to write csv content to file"))?;
        Ok(())
    }

    /// Overwrite virtual data's content into imported file
    ///
    /// * cache - whether to backup original file's content into temp directory
    pub fn overwrite_to_file(&self, page: &str, cache: bool) -> CedResult<bool> {
        if self.file.is_none() {
            return Ok(false);
        }

        let file = self.file.as_ref().unwrap();
        let csv = self.get_page_as_string(page)?;
        // Cache file into temp directory
        if cache {
            std::fs::copy(file, std::env::temp_dir().join("cache.csv"))
                .map_err(|err| CedError::io_error(err, "Failed to create cache for overwrite"))?;
        }
        std::fs::write(file, csv.as_bytes())
            .map_err(|err| CedError::io_error(err, "Failed to overwrite file with content"))?;
        Ok(true)
    }

    pub fn edit_cell(&mut self, page: &str, x: usize, y: usize, input: &str) -> CedResult<()> {
        self.get_page_data_mut(page)?
            .set_cell_from_string(x, y, input)?;
        Ok(())
    }

    pub fn edit_column(&mut self, page: &str, column: &str, input: &str) -> CedResult<()> {
        self.get_page_data_mut(page)?
            .set_column(column, Value::Text(input.to_owned()))?;
        Ok(())
    }

    pub fn edit_row(
        &mut self,
        page: &str,
        row_number: usize,
        input: &[Option<Value>],
    ) -> CedResult<()> {
        self.get_page_data_mut(page)?.edit_row(row_number, input)?;
        Ok(())
    }

    pub fn set_row(&mut self, page: &str, row_number: usize, input: &[Value]) -> CedResult<()> {
        self.get_page_data_mut(page)?.set_row(row_number, input)?;
        Ok(())
    }

    pub fn set_row_from_string(
        &mut self,
        page: &str,
        row_number: usize,
        input: &[impl AsRef<str>],
    ) -> CedResult<()> {
        self.get_page_data_mut(page)?.set_row(
            row_number,
            &input
                .iter()
                .map(|s| Value::Text(s.as_ref().to_owned()))
                .collect::<Vec<_>>(),
        )?;
        Ok(())
    }

    pub fn add_row(
        &mut self,
        page: &str,
        row_number: usize,
        values: Option<&[Value]>,
    ) -> CedResult<()> {
        self.get_page_data_mut(page)?
            .insert_row(row_number, values)?;
        Ok(())
    }

    pub fn add_row_from_strings(
        &mut self,
        page: &str,
        row_number: usize,
        src: &[impl AsRef<str>],
    ) -> CedResult<()> {
        let values = src
            .iter()
            .map(|a| Value::Text(a.as_ref().to_string()))
            .collect::<Vec<Value>>();
        self.add_row(page, row_number, Some(&values))?;
        Ok(())
    }

    pub fn add_column(
        &mut self,
        page: &str,
        column_number: usize,
        column_name: &str,
        column_type: ValueType,
        limiter: Option<ValueLimiter>,
        placeholder: Option<Value>,
    ) -> CedResult<()> {
        self.get_page_data_mut(page)?.insert_column_with_type(
            column_number,
            column_name,
            column_type,
            limiter,
            placeholder,
        )?;
        Ok(())
    }

    pub fn remove_row(&mut self, page: &str, row_number: usize) -> CedResult<bool> {
        Ok(self.get_page_data_mut(page)?.delete_row(row_number))
    }

    pub fn remove_column(&mut self, page: &str, column_number: usize) -> CedResult<()> {
        self.get_page_data_mut(page)?.delete_column(column_number)?;
        Ok(())
    }

    pub fn add_column_array(&mut self, page: &str, columns: &[impl AsRef<str>]) -> CedResult<()> {
        for col in columns {
            let column_count = self.get_page_data_mut(page)?.get_column_count();
            self.add_column(
                page,
                column_count,
                col.as_ref(),
                ValueType::Text,
                None,
                None,
            )?;
        }
        Ok(())
    }

    pub fn move_row(&mut self, page: &str, src: usize, target: usize) -> CedResult<()> {
        self.get_page_data_mut(page)?.move_row(src, target)?;
        Ok(())
    }

    pub fn move_column(&mut self, page: &str, src: usize, target: usize) -> CedResult<()> {
        self.get_page_data_mut(page)?.move_column(src, target)?;
        Ok(())
    }

    pub fn rename_column(&mut self, page: &str, column: &str, new_name: &str) -> CedResult<()> {
        let page = self.get_page_data_mut(page)?;
        if let Some(column) = page.try_get_column_index(column) {
            page.rename_column(column, new_name)?;
        } else {
            return Err(CedError::OutOfRangeError);
        }
        Ok(())
    }

    pub fn export_schema(&self, page: &str) -> CedResult<String> {
        let page = self.get_page_data(page)?;
        if page.is_array() {
            return Err(CedError::InvalidPageOperation(String::from(
                "Cannot export schmea from virtual array",
            )));
        }
        if !page.is_array() {
            // Sincie it is not an array, it is ok to unwrap
            Ok(page.get_data().unwrap().export_schema())
        } else {
            Err(CedError::InvalidPageOperation(
                "Cannot export schmea when csv is imported as array".to_string(),
            ))
        }
    }

    pub fn set_schema(&mut self, page: &str, path: impl AsRef<Path>, panic: bool) -> CedResult<()> {
        if self.get_page_data_mut(page)?.is_array() {
            return Err(CedError::InvalidPageOperation(
                "Cannot set schema in array mode".to_string(),
            ));
        }

        let content = std::fs::read_to_string(&path).map_err(|err| {
            CedError::io_error(
                err,
                &format!("Failed to import file \"{}\"", path.as_ref().display()),
            )
        })?;
        let mut content = content.lines();

        let header = content.next();
        if header.is_none() {
            return Err(CedError::InvalidRowData(
                "Given file does not have a header".to_string(),
            ));
        }

        let mut row = content.next();
        while let Some(row_src) = row {
            let row_args = dcsv::utils::csv_row_to_vector(row_src, None, false);
            let limiter = ValueLimiter::from_line(&row_args[1..].to_vec())?;
            self.set_limiter(page, &row_args[0], &limiter, panic)?;
            row = content.next();
        }
        Ok(())
    }

    pub fn set_limiter(
        &mut self,
        page: &str,
        column: &str,
        limiter: &ValueLimiter,
        panic: bool,
    ) -> CedResult<()> {
        if self.get_page_data(page)?.is_array() {
            return Err(CedError::InvalidPageOperation(String::from(
                "Cannot set limiter for virtual array",
            )));
        }
        let column = self
            .get_page_data_mut(page)?
            .try_get_column_index(column)
            .ok_or_else(|| CedError::InvalidColumn(format!("{} is not a valid column", column)))?;
        self.get_page_data_mut(page)?
            .set_limiter(column, limiter, panic)?;
        Ok(())
    }

    // <PRESETS>
    //
    #[cfg(feature = "cli")]
    pub(crate) fn configure_preset(&mut self, use_defualt: bool) -> CedResult<()> {
        self.preset = Preset::new(use_defualt)?;
        Ok(())
    }

    #[cfg(feature = "cli")]
    pub(crate) fn set_limiter_from_preset(
        &mut self,
        page: &str,
        column: &str,
        preset_name: &str,
        panic: bool,
    ) -> CedResult<()> {
        let preset = self.preset.get(preset_name).cloned();
        if let Some(limiter) = preset {
            self.set_limiter(page, column, &limiter, panic)?;
        }
        Ok(())
    }

    // <MISC>
    pub fn get_row_count(&self, page: &str) -> CedResult<usize> {
        Ok(self.get_page_data(page)?.get_row_count())
    }

    pub fn get_column_count(&self, page: &str) -> CedResult<usize> {
        Ok(self.get_page_data(page)?.get_column_count())
    }

    /// Get last row index
    pub fn last_row_index(&self, page: &str) -> CedResult<usize> {
        Ok(self.get_page_data(page)?.get_row_count().max(1) - 1)
    }

    /// Get last column index
    pub fn last_column_index(&self, page: &str) -> CedResult<usize> {
        Ok(self.get_page_data(page)?.get_column_count().max(1) - 1)
    }

    /// Get virtual data as string form
    pub fn get_page_as_string(&self, page: &str) -> CedResult<String> {
        Ok(self.get_page_data(page)?.to_string())
    }

    pub fn get_cell(&self, page: &str, row: usize, column: usize) -> CedResult<Option<&Value>> {
        Ok(self.get_page_data(page)?.get_cell(row, column))
    }

    pub fn get_column(&self, page: &str, column_index: usize) -> CedResult<Option<&Column>> {
        let page = self.get_page_data(page)?;
        Ok(page.get_columns().get(column_index))
    }

    pub fn get_column_by_name(&self, page: &str, column_name: &str) -> CedResult<Option<&Column>> {
        let page = self.get_page_data(page)?;
        Ok(match page.try_get_column_index(column_name) {
            Some(index) => page.get_columns().get(index),
            None => None,
        })
    }
}
