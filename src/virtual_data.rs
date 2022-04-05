use crate::error::{CedError, CedResult};
use crate::value::{Value, ValueLimiter, ValueType};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct VirtualData {
    pub(crate) columns: Vec<Column>,
    pub(crate) rows: Vec<Row>,
}

impl std::fmt::Display for VirtualData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut csv_src = String::new();
        let column_row = self
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<&str>>()
            .join(",")
            + "\n";
        csv_src.push_str(&column_row);

        let columns = self
            .columns
            .iter()
            .map(|col| col.name.as_str())
            .collect::<Vec<&str>>();
        for row in &self.rows {
            let row_value = columns
                .iter()
                .map(|name| {
                    row.get_value(name)
                        .unwrap_or(&Value::Text(String::new()))
                        .to_string()
                })
                .collect::<Vec<String>>()
                .join(",")
                + "\n";

            csv_src.push_str(&row_value);
        }
        // Remove trailing newline
        csv_src.pop();
        write!(f, "{}", csv_src)
    }
}

impl VirtualData {
    pub fn new() -> Self {
        Self {
            columns: vec![],
            rows: vec![],
        }
    }

    pub fn set_data_from_string(&mut self, x: usize, y: usize, value: &str) -> CedResult<()> {
        let key_column = self.get_column_if_valid(x, y)?;
        match key_column.column_type {
            ValueType::Text => self.set_data(x, y, Value::Text(value.to_string())),
            ValueType::Number => self.set_data(
                x,
                y,
                Value::Number(value.parse().map_err(|_| {
                    CedError::InvalidCellData(format!(
                        "Given value is \"{}\" which is not a number",
                        value
                    ))
                })?),
            ),
        }
    }

    pub fn move_row(&mut self, src: usize, target: usize) -> CedResult<()> {
        let row_count = self.get_row_count();
        if src >= row_count || target >= row_count {
            return Err(CedError::OutOfRangeError);
        }

        let move_direction = src.cmp(&target);
        match move_direction {
            // Go left
            Ordering::Greater => {
                let mut index = src;
                let mut next = index - 1;
                while next >= target {
                    self.rows.swap(index, next);

                    // Usize specific check code
                    if next == 0 {
                        break;
                    }

                    // Update index values
                    index -= 1;
                    next -= 1;
                }
            }
            Ordering::Less => {
                // Go right
                let mut index = src;
                let mut next = index + 1;
                while next <= target {
                    self.rows.swap(index, next);

                    // Update index values
                    index += 1;
                    next += 1;
                }
            }
            Ordering::Equal => (),
        }
        Ok(())
    }

    pub fn move_column(&mut self, src: usize, target: usize) -> CedResult<()> {
        let column_count = self.get_column_count();
        if src >= column_count || target >= column_count {
            return Err(CedError::OutOfRangeError);
        }

        let move_direction = src.cmp(&target);
        match move_direction {
            // Go left
            Ordering::Greater => {
                let mut index = src;
                let mut next = index - 1;
                while next >= target {
                    self.columns.swap(index, next);

                    // Usize specific check code
                    if next == 0 {
                        break;
                    }

                    // Update index values
                    index -= 1;
                    next -= 1;
                }
            }
            Ordering::Less => {
                // Go right
                let mut index = src;
                let mut next = index + 1;
                while next <= target {
                    self.columns.swap(index, next);

                    // Update index values
                    index += 1;
                    next += 1;
                }
            }
            Ordering::Equal => (),
        }
        Ok(())
    }

    pub fn rename_column(&mut self, column: &str, new_name: &str) -> CedResult<()> {
        let column_index = self.try_get_column_index(column);

        if let None = column_index {
            return Err(CedError::OutOfRangeError);
        }

        let previous = self.columns[column_index.unwrap()].rename(new_name);
        for row in &mut self.rows {
            row.rename_column(&previous, new_name);
        }
        Ok(())
    }

    // TODO
    // 1. Check limiter
    // 2. Check if value exists
    pub fn set_column(&mut self, column: &str, value: Value) -> CedResult<()> {
        let column_index = self.try_get_column_index(column);
        if let None = column_index {
            return Err(CedError::OutOfRangeError);
        }
        let column = &self.columns[column_index.unwrap()].name;
        for row in &mut self.rows {
            row.update_value(column, value.clone());
        }
        Ok(())
    }

    // TODO
    // 1. Check limiter
    // 2. Check if value exists
    pub fn set_row(&mut self, row_number: usize, values: Vec<Value>) -> CedResult<()> {
        // Row's value doesn't match length of columns
        if values.len() != self.get_column_count() {
            return Err(CedError::InsufficientRowData);
        }
        // Invalid cooridnate
        if !self.is_valid_cell_coordinate(row_number, 0) {
            return Err(CedError::OutOfRangeError);
        }

        let col_value_iter = self.columns.iter().zip(values.iter());

        for (col, value) in col_value_iter.clone() {
            // Early return if doesn't qualify a single element
            if !col.limiter.qualify(value) {
                return Err(CedError::InvalidRowData(format!(
                    "{} doesn't qualify {}'s limiter",
                    value.to_string(),
                    col.name
                )));
            }
        }

        let row = self.rows.get_mut(row_number).unwrap();
        for (col, value) in col_value_iter {
            row.update_value(&col.name, value.clone())
        }

        Ok(())
    }

    /// Set data by coordinate
    pub fn set_data(&mut self, x: usize, y: usize, value: Value) -> CedResult<()> {
        let name = self.get_column_if_valid(x, y)?.name.to_owned();

        self.is_valid_column_data(y, &value)?;
        self.rows[x].update_value(&name, value);

        Ok(())
    }

    // THis should insert row with given column limiters
    pub fn insert_row(&mut self, row: usize, source: Option<&Vec<Value>>) -> CedResult<()> {
        let mut new_row = Row::new();
        if let Some(source) = source {
            self.check_row_length(source)?;
            self.columns
                .iter()
                .zip(source.iter())
                .for_each(|(col, v)| new_row.insert_value(&col.name, v.clone()));
        } else {
            for col in &self.columns {
                new_row.insert_value(&col.name, col.get_default_value());
            }
        }
        self.rows.insert(row, new_row);
        Ok(())
    }

    pub fn delete_row(&mut self, row: usize) -> Option<Row> {
        let row_count = self.get_row_count();
        if row_count == 0 || row_count < row {
            return None;
        }
        Some(self.rows.remove(row))
    }

    pub fn insert_column(
        &mut self,
        column: usize,
        column_name: &str,
        column_type: ValueType,
        limiter: Option<ValueLimiter>,
    ) -> CedResult<()> {
        if let Some(_) = self.try_get_column_index(column_name) {
            return Err(CedError::InvalidColumn(format!(
                "Cannot add existing column or number named column"
            )));
        }
        if let Ok(_) = column_name.parse::<isize>() {
            return Err(CedError::InvalidColumn(format!(
                "Cannot add number named column"
            )));
        }
        let new_column = Column::new(column_name, column_type, limiter);
        let default_value = new_column.get_default_value();
        for row in &mut self.rows {
            row.insert_value(&new_column.name, default_value.clone());
        }
        self.columns.insert(column, new_column);
        Ok(())
    }

    pub fn delete_column(&mut self, column: usize) -> CedResult<()> {
        let name = self.get_column_if_valid(0, column)?.name.to_owned();

        for row in &mut self.rows {
            row.remove_value(&name);
        }

        self.columns.remove(column);

        // If column is empty, drop all rows
        if self.get_column_count() == 0 {
            self.rows = vec![];
        }

        Ok(())
    }

    pub fn set_limiter(&mut self, column: usize, limiter: ValueLimiter) -> CedResult<()> {
        self.columns[column].set_limiter(limiter);
        Ok(())
    }

    // <DRY>
    pub(crate) fn try_get_column_index(&self, src: &str) -> Option<usize> {
        let column_index = match src.parse::<usize>() {
            Err(_) => self.columns.iter().position(|c| c.name == src),
            Ok(index) => Some(index),
        };
        column_index
    }

    fn is_valid_cell_coordinate(&self, x: usize, y: usize) -> bool {
        if x < self.get_row_count() {
            if y < self.get_column_count() {
                return true;
            }
        }

        false
    }

    fn get_column_if_valid(&self, x: usize, y: usize) -> CedResult<&Column> {
        if !self.is_valid_cell_coordinate(x, y) {
            return Err(CedError::OutOfRangeError);
        }
        let key_column = self.columns.get(y).unwrap();
        Ok(key_column)
    }

    /// Check if given value corresponds to column limiter
    fn is_valid_column_data(&self, column: usize, value: &Value) -> CedResult<()> {
        if let Some(col) = self.columns.get(column) {
            if col.limiter.qualify(value) {
                Ok(())
            } else {
                return Err(CedError::InvalidCellData(format!(
                    "Given cell data failed to match limiter's restriction",
                )));
            }
        } else {
            return Err(CedError::InvalidRowData(format!(
                "Given column number \"{}\" doesn't exist",
                column
            )));
        }
    }

    /// Check if given values' length match column's legnth
    fn check_row_length(&self, values: &Vec<Value>) -> CedResult<()> {
        match self.get_column_count().cmp(&values.len()) {
            Ordering::Equal => (),
            Ordering::Less | Ordering::Greater => {
                return Err(CedError::InvalidRowData(format!(
                    r#"Given row length is "{}" while columns length is "{}""#,
                    values.len(),
                    self.get_column_count()
                )))
            }
        }
        Ok(())
    }

    // </DRY>

    // <EXT>
    pub fn get_row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn get_column_count(&self) -> usize {
        self.columns.len()
    }

    /// Drop all data
    pub fn drop(&mut self) {
        self.columns.clear();
        self.rows.clear();
    }

    // </EXT>
}

#[derive(Clone, Debug)]
pub struct Column {
    pub(crate) name: String,
    pub(crate) column_type: ValueType,
    pub(crate) limiter: ValueLimiter,
}

impl Column {
    pub fn new(name: &str, column_type: ValueType, limiter: Option<ValueLimiter>) -> Self {
        Self {
            name: name.to_string(),
            column_type,
            limiter: limiter.unwrap_or(ValueLimiter::default()),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_column_type(&self) -> &ValueType {
        &self.column_type
    }

    pub(crate) fn rename(&mut self, new_name: &str) -> String {
        std::mem::replace(&mut self.name, new_name.to_string())
    }

    pub(crate) fn set_limiter(&mut self, limiter: ValueLimiter) {
        self.column_type = limiter.get_type();
        self.limiter = limiter;
    }

    pub fn get_default_value(&self) -> Value {
        // has default
        if let Some(def) = self.limiter.get_default() {
            return def.clone();
        }

        // has variant
        let variant = self.limiter.get_variant();
        if let Some(vec) = variant {
            if vec.len() != 0 {
                return vec[0].clone();
            }
        }

        // Construct new default value
        match self.column_type {
            ValueType::Number => Value::Number(0),
            ValueType::Text => Value::Text(String::new()),
        }
    }
}

#[derive(Clone)]
pub struct Row {
    values: HashMap<String, Value>,
}

impl Row {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub(crate) fn rename_column(&mut self, name: &str, new_name: &str) {
        let previous = self.values.remove(name);

        if let Some(prev) = previous {
            self.values.insert(new_name.to_string(), prev);
        }
    }

    pub fn insert_value(&mut self, key: &str, value: Value) {
        self.values.insert(key.to_string(), value);
    }

    pub fn get_value(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    pub fn update_value(&mut self, key: &str, value: Value) {
        *self.values.get_mut(key).unwrap() = value;
    }

    pub(crate) fn remove_value(&mut self, key: &str) {
        self.values.remove(key);
    }
}
