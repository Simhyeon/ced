use crate::value::{Value, ValueType, ValueLimiter};
use crate::error::{CedError, CedResult};
use std::collections::HashMap;

pub(crate) struct VirtualData {
    columns: Vec<Column>,
    rows: Vec<Row>,
}

impl VirtualData {
    pub fn set_data_from_raw(&mut self, x: usize, y: usize, value : &str)  -> CedResult<()> {
        let key_column = self.get_column_if_valid(x, y)?;
        match key_column.column_type {
            ValueType::Text => self.set_data(x,y,Value::Text(value.to_string())),
            ValueType::Number => self.set_data(x,y,
                Value::Number(
                    value.parse().
                    map_err(|_| CedError::InvalidCellData)?
                )
            ),
        }
    }

    /// Set data by coordinate
    pub fn set_data(&mut self, x: usize, y: usize, value : Value) -> CedResult<()>  {
        let id = self.get_column_if_valid(x, y)?.id.to_string();

        if self.is_valid_column_data(y, &value) {
            self.rows[x].update_value(&id, value);
        } else { 
            return Err(CedError::InvalidCellData); 
        }

        Ok(())
    }

    // THis should insert row with given column limiters
    pub fn insert_row(&mut self, row: usize) {
        let mut new_row = Row::new();
        for col in &self.columns {
            new_row.insert_value(&col.id, col.get_default_value());
        }
        self.rows.insert(row, new_row);
    }

    pub fn delete_row(&mut self, row: usize) {
        self.rows.remove(row);
    }

    pub fn insert_column(&mut self, column : usize, column_name: &str, column_type: ValueType, limiter: Option<ValueLimiter>) {
        let new_column = Column::new(column_name,column_type, limiter);
        let default_value = new_column.get_default_value();
        for row in &mut self.rows {
            row.insert_value(&new_column.id, default_value.clone());
        }
        self.columns.insert(column, new_column);
    }

    pub fn delete_column(&mut self, column : usize) -> CedResult<()> {
        let id = self.get_column_if_valid(0, column)?.id.to_string();

        for row in &mut self.rows {
            row.remove_value(&id);
        }
        self.columns.remove(column);
        Ok(())
    }
    
    pub fn is_valid_cell_coordinate(&self, x:usize,y:usize) -> bool {
        if x >= 0 && x <= self.get_row_count() {
            if y >= 0 && y <= self.get_column_count() {
                return true;
            }
        }

        false
    }

    pub fn get_column_if_valid(&self, x: usize, y: usize) -> CedResult<&Column> {
        if !self.is_valid_cell_coordinate(x, y) {
             return Err(CedError::OutOfRangeError); 
        }
        let key_column = self.columns.get(y).unwrap();
        Ok(key_column)
    }

    // TODO
    pub fn is_valid_column_data(&self, column: usize,value: &Value) -> bool {
        false
    }


    pub fn get_row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn get_column_count(&self) -> usize {
        self.columns.len()
    }
}

pub struct Column {
    name       : String,
    id         : String,
    column_type: ValueType,
    limiter    : ValueLimiter,
}

impl Column {
    pub fn new(name: &str, column_type: ValueType, limiter: Option<ValueLimiter>) -> Self {
        Self {
            name: name.to_string(),
            id: String::new(),
            column_type,
            limiter: limiter.unwrap_or(ValueLimiter::default()),
        }
    }

    pub fn set_limiter(&mut self, limiter: ValueLimiter) {
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
            ValueType::Number => { Value::Number(0) }
            ValueType::Text => {
                let mut default_string = String::new();
                if let Some(pre) = self.limiter.get_prefix() {
                    default_string.push_str(pre);
                }

                if let Some(post) = self.limiter.get_postfix() {
                    default_string.push_str(" ");
                    default_string.push_str(post);
                }

                Value::Text(default_string)
            }
        }
    }
}

pub struct Row {
    values : HashMap<String,Value>,
}

impl Row {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
    
    pub fn insert_value(&mut self, key: &str, value: Value) {
        self.values.insert(key.to_string(),value);
    }

    pub fn get_value(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    pub fn update_value(&mut self, key: &str, value: Value) {
        *self.values.get_mut(key).unwrap() = value;
    } 

    pub fn remove_value(&mut self, key: &str) {
        self.values.remove(key);
    }
}
