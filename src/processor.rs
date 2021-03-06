/// Processor is a main struct for csv editing.
///
/// Generic workflow of ced processor is followed.
///
/// - Add a page(csv value) to a processor
/// - Use processor api with page names
/// - Discard or save modified data to a file
///
/// * Usage
/// ```rust
/// use ced::Processor;
/// let mut processor = Processor::new();
///
/// processor.import_from_file("test.csv", true, None, false).unwrap();
///
/// // Get current cursor(page_name) for later uses
/// let page_name = processor.get_cursor().unwrap();
///
/// // Processor can hold multiple pages and needs page_name for every operation to work on the
/// // page
/// processor.add_row_from_string_array(&page_name, processor.last_row_index(&page_name)?, &["a","b"]).unwrap();
///
/// processor.overwrite_to_file(&page_name,true).unwrap();
/// ```
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[cfg(feature = "cli")]
use crate::cli::preset::Preset;
use crate::error::{CedError, CedResult};
use crate::page::Page;
use crate::utils;
use dcsv::Column;
use dcsv::{Value, ValueLimiter, ValueType};
use std::collections::HashMap;

/// Csv processor
///
/// Processor has multiple pages which can be accessed with page_name. Processor has currently
/// selected page which name can be accessed with ```get_cursor``` method.
pub struct Processor {
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
    /// Create empty processor
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
            cursor: None,
            print_logs: true,
            #[cfg(feature = "cli")]
            preset: Preset::empty(),
            #[cfg(feature = "cli")]
            no_loop: false,
        }
    }

    /// Change current cusor(page)
    ///
    /// This doesn't affect page itself but change cursor.
    /// * This returns boolean value whether change succeeded or not
    pub fn change_cursor(&mut self, page_name: &str) -> bool {
        if !self.pages.contains_key(page_name) {
            false
        } else {
            self.cursor = Some(page_name.to_owned());
            true
        }
    }

    /// Get current cursor (page_name)
    pub fn get_cursor(&self) -> Option<String> {
        self.cursor.as_ref().map(|s| s.to_string())
    }

    /// Add a new page
    ///
    /// # Args
    ///
    /// * page : Page name to create
    /// * data : Csv data to store inside a page
    /// * has_header : Whether csv data has header or not.
    /// * line_ending : Optional line_ending configuration.
    /// * raw_mode : This decides whether page be data or array
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

    /// Remove page with given name
    ///
    /// This doesn't panic and silent do nothing if page name is non-existent
    pub fn remove_page(&mut self, page_name: &str) {
        self.pages.remove_entry(page_name);
    }

    /// Check if processor contains a page
    pub fn contains_page(&self, page: &str) -> bool {
        self.pages.contains_key(page)
    }

    /// Try get page data but panic if cursor is empty
    ///
    /// # Return
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

    /// Try get page data but panic if page is non-existent
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

    /// Drop all data from processor
    pub fn drop_pages(&mut self) -> CedResult<()> {
        self.pages.clear();
        self.cursor = None;
        Ok(())
    }

    /// Import file content as page
    ///
    /// This will drop the page if given page name already exists.
    ///
    /// # Args
    ///
    /// * path: File path to import from
    /// * has_header : Whether csv file has header or not
    /// * line_ending : Optional line_ending of csv
    /// * raw_mode : Whether imported as data or array
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

        self.add_page(page_name, &content, has_header, line_ending, raw_mode)?;

        // Set source file because it was imported from file
        self.pages
            .get_mut(page_name)
            .unwrap()
            .set_source_file(path.as_ref().to_owned());
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

    /// Overwrite virtual data's content into a imported file
    ///
    /// * cache : whether to backup original file's content into temp directory
    pub fn overwrite_to_file(&self, page: &str, cache: bool) -> CedResult<bool> {
        let page = self.get_page_data(page)?;
        let file = page.source_file.as_ref();
        if file.is_none() {
            return Ok(false);
        }

        let file = file.unwrap();
        let csv = page.to_string();
        // Cache file into temp directory
        if cache {
            std::fs::copy(file, std::env::temp_dir().join("cache.csv"))
                .map_err(|err| CedError::io_error(err, "Failed to create cache for overwrite"))?;
        }
        std::fs::write(file, csv.as_bytes())
            .map_err(|err| CedError::io_error(err, "Failed to overwrite file with content"))?;
        Ok(true)
    }

    /// Edit a cell by given coordinate
    pub fn edit_cell(&mut self, page: &str, x: usize, y: usize, input: &str) -> CedResult<()> {
        self.get_page_data_mut(page)?
            .set_cell_from_string(x, y, input)?;
        Ok(())
    }

    /// Edit a column by given coordinate
    ///
    /// This overwrite all column values with given input
    pub fn edit_column(&mut self, page: &str, column: &str, input: &str) -> CedResult<()> {
        self.get_page_data_mut(page)?
            .set_column(column, Value::Text(input.to_owned()))?;
        Ok(())
    }

    /// Edit a row with values
    ///
    /// This assumes given input accords with order of a target record.
    ///
    /// # Args
    ///
    /// * page : Page name
    /// * row_index : Target row
    /// * input : Inputs are array of options. Some will overwrite and none will not.
    pub fn edit_row(
        &mut self,
        page: &str,
        row_index: usize,
        input: &[Option<Value>],
    ) -> CedResult<()> {
        self.get_page_data_mut(page)?.edit_row(row_index, input)?;
        Ok(())
    }

    /// Set a row with given values
    ///
    /// This assumes given input accords with order of a target record.
    /// This method overwrite an entire row with given values.
    pub fn set_row(&mut self, page: &str, row_index: usize, input: &[Value]) -> CedResult<()> {
        self.get_page_data_mut(page)?.set_row(row_index, input)?;
        Ok(())
    }

    /// Set a row with given string array
    ///
    /// This assumes given input accords with order of a target record.
    /// This method overwrite an entire row with given values.
    pub fn set_row_from_string_array(
        &mut self,
        page: &str,
        row_index: usize,
        input: &[impl AsRef<str>],
    ) -> CedResult<()> {
        self.get_page_data_mut(page)?.set_row(
            row_index,
            &input
                .iter()
                .map(|s| Value::Text(s.as_ref().to_owned()))
                .collect::<Vec<_>>(),
        )?;
        Ok(())
    }

    /// Add a new row
    ///
    /// This assumes given input accords with order of a target record.
    ///
    /// # Args
    ///
    /// * page: Target page
    /// * row_index : Target row
    /// * values : Option. "None" will converted as default values.
    pub fn add_row(
        &mut self,
        page: &str,
        row_index: usize,
        values: Option<&[Value]>,
    ) -> CedResult<()> {
        self.get_page_data_mut(page)?
            .insert_row(row_index, values)?;
        Ok(())
    }

    /// Add a new row but from array of strings
    ///
    /// This assumes given input accords with order of a target record.
    ///
    /// # Args
    ///
    /// * page: Target page
    /// * row_index : Target row
    /// * values : Option. "None" will converted as default values.
    pub fn add_row_from_string_array(
        &mut self,
        page: &str,
        row_index: usize,
        src: &[impl AsRef<str>],
    ) -> CedResult<()> {
        let values = src
            .iter()
            .map(|a| Value::Text(a.as_ref().to_string()))
            .collect::<Vec<Value>>();
        self.add_row(page, row_index, Some(&values))?;
        Ok(())
    }

    /// Add a new column into a page
    pub fn add_column(
        &mut self,
        page: &str,
        column_index: usize,
        column_name: &str,
        column_type: ValueType,
        limiter: Option<ValueLimiter>,
        placeholder: Option<Value>,
    ) -> CedResult<()> {
        self.get_page_data_mut(page)?.insert_column_with_type(
            column_index,
            column_name,
            column_type,
            limiter,
            placeholder,
        )?;
        Ok(())
    }

    /// Remove a row from a page
    pub fn remove_row(&mut self, page: &str, row_index: usize) -> CedResult<bool> {
        Ok(self.get_page_data_mut(page)?.delete_row(row_index))
    }

    /// Remove a column from a page
    pub fn remove_column(&mut self, page: &str, column_index: usize) -> CedResult<()> {
        self.get_page_data_mut(page)?.delete_column(column_index)?;
        Ok(())
    }

    /// Add columns into a page
    ///
    /// This method dosn't require any column configurators
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

    /// Move a rom from an index to a target index
    pub fn move_row(&mut self, page: &str, src: usize, target: usize) -> CedResult<()> {
        self.get_page_data_mut(page)?.move_row(src, target)?;
        Ok(())
    }

    /// Move a column from an index to a target index
    pub fn move_column(&mut self, page: &str, src: usize, target: usize) -> CedResult<()> {
        self.get_page_data_mut(page)?.move_column(src, target)?;
        Ok(())
    }

    /// Rename a column into a new name
    pub fn rename_column(&mut self, page: &str, column: &str, new_name: &str) -> CedResult<()> {
        let page = self.get_page_data_mut(page)?;
        if let Some(column) = page.try_get_column_index(column) {
            page.rename_column(column, new_name)?;
        } else {
            return Err(CedError::OutOfRangeError);
        }
        Ok(())
    }

    /// Export page's schema
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

    /// Apply schema into a given page
    ///
    /// # Args
    ///
    /// * page : Page name
    /// * path : Schema file path
    /// * panic : Whether to panic if current value fails to qualify schema. If not every
    /// unqualified values are overwritten to default qualifying values.
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

    /// Set a limiter to a column
    ///
    /// # Args
    ///
    /// * page : Page name
    /// * column : Target column name(index)
    /// * limiter : A limiter to apply to column
    /// * panic : Whether to panic if current value fails to qualify liimter. If not, every
    /// unqualified values are overwritten to default qualifying values.
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

    /// Get cell from page
    ///
    /// This fails when page or coordinate doesn't exist
    pub fn get_cell(
        &self,
        page: &str,
        row_index: usize,
        column_index: usize,
    ) -> CedResult<Option<&Value>> {
        Ok(self.get_page_data(page)?.get_cell(row_index, column_index))
    }

    /// Get column from page
    ///
    /// This fails when either page or column doesn't exist
    pub fn get_column(&self, page: &str, column_index: usize) -> CedResult<Option<&Column>> {
        let page = self.get_page_data(page)?;
        Ok(page.get_columns().get(column_index))
    }

    /// Get column from page by name
    ///
    /// This fails when either page or column doesn't exist
    pub fn get_column_by_name(&self, page: &str, column_name: &str) -> CedResult<Option<&Column>> {
        let page = self.get_page_data(page)?;
        Ok(match page.try_get_column_index(column_name) {
            Some(index) => page.get_columns().get(index),
            None => None,
        })
    }
}
