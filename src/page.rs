use std::path::PathBuf;

use dcsv::{Column, VCont, Value, ValueLimiter, ValueType, VirtualArray, VirtualData};

use crate::{CedError, CedResult};

#[derive(Clone)]
pub(crate) struct Page {
    pub(crate) source_file: Option<PathBuf>,
    content: PageContent,
}

#[derive(Clone)]
pub(crate) enum PageContent {
    Data(VirtualData),
    Array(VirtualArray),
}

impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let src = match &self.content {
            PageContent::Data(data) => data.to_string(),
            PageContent::Array(array) => array.to_string(),
        };
        write!(f, "{}", src)
    }
}

impl Page {
    pub fn set_source_file(&mut self, source_file: PathBuf) {
        self.source_file.replace(source_file);
    }

    pub fn new_data(data: VirtualData) -> Self {
        Self {
            source_file: None,
            content: PageContent::Data(data),
        }
    }

    pub fn new_array(array: VirtualArray) -> Self {
        Self {
            source_file: None,
            content: PageContent::Array(array),
        }
    }
    pub fn move_column(&mut self, src_index: usize, target_index: usize) -> CedResult<()> {
        match &mut self.content {
            PageContent::Data(data) => data.move_column(src_index, target_index)?,
            PageContent::Array(array) => array.move_column(src_index, target_index)?,
        }
        Ok(())
    }

    pub fn move_row(&mut self, src_index: usize, target_index: usize) -> CedResult<()> {
        match &mut self.content {
            PageContent::Data(data) => data.move_row(src_index, target_index)?,
            PageContent::Array(array) => array.move_row(src_index, target_index)?,
        }
        Ok(())
    }

    pub fn delete_row(&mut self, row_index: usize) -> bool {
        match &mut self.content {
            PageContent::Data(data) => data.delete_row(row_index),
            PageContent::Array(array) => array.delete_row(row_index),
        }
    }

    pub fn delete_column(&mut self, column_index: usize) -> CedResult<()> {
        match &mut self.content {
            PageContent::Data(data) => data.delete_column(column_index)?,
            PageContent::Array(array) => array.delete_column(column_index)?,
        }
        Ok(())
    }

    /// Insert column with type
    ///
    /// This doesn't add any type in array content
    pub fn insert_column_with_type(
        &mut self,
        column_index: usize,
        column_name: &str,
        column_type: ValueType,
        limiter: Option<ValueLimiter>,
        placeholder: Option<Value>,
    ) -> CedResult<()> {
        match &mut self.content {
            PageContent::Data(data) => data.insert_column_with_type(
                column_index,
                column_name,
                column_type,
                limiter,
                placeholder,
            )?,
            PageContent::Array(array) => array.insert_column(column_index, column_name)?,
        }
        Ok(())
    }

    pub fn insert_row(&mut self, row_index: usize, source: Option<&[Value]>) -> CedResult<()> {
        match &mut self.content {
            PageContent::Data(data) => data.insert_row(row_index, source)?,
            PageContent::Array(array) => array.insert_row(row_index, source)?,
        }
        Ok(())
    }

    pub fn edit_row(&mut self, row_index: usize, values: &[Option<Value>]) -> CedResult<()> {
        match &mut self.content {
            PageContent::Data(data) => data.edit_row(row_index, values)?,
            PageContent::Array(array) => array.edit_row(row_index, values)?,
        }
        Ok(())
    }

    pub fn rename_column(&mut self, column_index: usize, new_name: &str) -> CedResult<()> {
        match &mut self.content {
            PageContent::Data(data) => data.rename_column(column_index, new_name)?,
            PageContent::Array(array) => array.rename_column(column_index, new_name)?,
        }
        Ok(())
    }

    pub fn get_data(&self) -> Option<&VirtualData> {
        match &self.content {
            PageContent::Data(data) => Some(data),
            PageContent::Array(_) => None,
        }
    }

    pub fn is_array(&self) -> bool {
        match self.content {
            PageContent::Data(_) => false,
            PageContent::Array(_) => true,
        }
    }
    pub fn set_limiter(
        &mut self,
        column: usize,
        limiter: &ValueLimiter,
        panic: bool,
    ) -> CedResult<()> {
        match &mut self.content {
            PageContent::Data(data) => data.set_limiter(column, limiter, panic)?,
            PageContent::Array(_) => {}
        };
        Ok(())
    }

    pub fn set_row(&mut self, row_index: usize, values: &[Value]) -> CedResult<()> {
        match &mut self.content {
            PageContent::Data(data) => data.set_row(row_index, values)?,
            PageContent::Array(array) => array.set_row(row_index, values)?,
        }
        Ok(())
    }

    pub fn set_column(&mut self, column: &str, value: Value) -> CedResult<()> {
        match &mut self.content {
            PageContent::Data(data) => match data.try_get_column_index(column) {
                Some(index) => data.set_column(index, value)?,
                None => {
                    return Err(CedError::InvalidColumn(format!(
                        "Cannot set a column \"{}\", which doesn't exist",
                        column
                    )));
                }
            },
            PageContent::Array(array) => {
                let column = column.parse::<usize>().map_err(|_| {
                    return CedError::InvalidColumn(format!(
                        "Given column index \"{}\"is not a valid integer ",
                        column
                    ));
                })?;
                array.set_column(column, Value::Text(value.to_string()))?
            }
        };
        Ok(())
    }

    pub fn try_get_column_index(&self, src: &str) -> Option<usize> {
        match &self.content {
            PageContent::Data(data) => data.try_get_column_index(src),
            PageContent::Array(array) => {
                if let Ok(num) = src.parse::<usize>() {
                    if array.columns.get(num).is_some() {
                        return Some(num);
                    }
                }
                None
            }
        }
    }

    pub fn get_rows(&self) -> Vec<Vec<&Value>> {
        match &self.content {
            PageContent::Data(data) => {
                let mut values_outer = vec![];
                for row in &data.rows {
                    // THis operation cannot fail
                    let values_inner = row.to_vector(&data.columns).unwrap();
                    values_outer.push(values_inner);
                }
                values_outer
            }
            // Though this is very archaic... single entry is hard to achive
            PageContent::Array(array) => array
                .rows
                .iter()
                .map(|row| row.iter().collect::<Vec<_>>())
                .collect::<Vec<_>>(),
        }
    }

    pub fn get_columns(&self) -> &Vec<Column> {
        match &self.content {
            PageContent::Data(data) => &data.columns,
            PageContent::Array(array) => &array.columns,
        }
    }

    pub fn get_row_count(&self) -> usize {
        match &self.content {
            PageContent::Data(data) => data.rows.len(),
            PageContent::Array(array) => array.rows.len(),
        }
    }

    pub fn get_column_count(&self) -> usize {
        match &self.content {
            PageContent::Data(data) => data.columns.len(),
            PageContent::Array(array) => array.columns.len(),
        }
    }
    pub fn get_cell(&self, x: usize, y: usize) -> Option<&dcsv::Value> {
        match &self.content {
            PageContent::Data(data) => data.get_cell(x, y),
            PageContent::Array(array) => array.get_cell(x, y),
        }
    }

    pub fn set_cell_from_string(&mut self, x: usize, y: usize, value: &str) -> CedResult<()> {
        match &mut self.content {
            PageContent::Data(data) => data.set_cell_from_string(x, y, value)?,
            PageContent::Array(array) => array.set_cell(x, y, Value::Text(value.to_string()))?,
        }
        Ok(())
    }

    pub fn get_row_as_string(&self, row_index: usize) -> CedResult<String> {
        let string = match &self.content {
            PageContent::Data(data) => {
                if let Some(row) = data.rows.get(row_index) {
                    row.to_string(&data.columns)?
                } else {
                    return Err(CedError::OutOfRangeError);
                }
            }
            PageContent::Array(array) => {
                if let Some(row) = array.rows.get(row_index) {
                    row.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                } else {
                    return Err(CedError::OutOfRangeError);
                }
            }
        };
        Ok(string)
    }
}

//match self.content {
//PageContent::Data(data),
//PageContent::Array(array),
//}
