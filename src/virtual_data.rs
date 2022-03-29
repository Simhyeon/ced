use crate::value::{Value, ValueType, ValueLimiter};
use crate::error::{CedError, CedResult};
use std::cmp::Ordering;
use std::collections::HashMap;

pub(crate) struct VirtualData {
    pub(crate) columns: Vec<Column>,
    pub(crate) rows: Vec<Row>,
}

impl std::fmt::Display for VirtualData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut csv_src = String::new();
        let column_row = self.columns.iter().map(|c| c.name.as_str()).collect::<Vec<&str>>().join(",") + "\n";
        csv_src.push_str(&column_row);

        let columns = self.columns.iter().map(|col| col.name.as_str()).collect::<Vec<&str>>();
        for row in &self.rows {
            let row_value = columns.iter().map(|name| {
                row.get_value(name)
                    .unwrap_or(&Value::Text(String::new()))
                    .to_string()
            }).collect::<Vec<String>>()
            .join(",") + "\n";

            csv_src.push_str(&row_value);
        }
        write!(f,"{}",csv_src)
    }
}

impl VirtualData {
    pub fn new() -> Self {
        Self {
            columns: vec![],
            rows: vec![],
        }
    }
    pub fn set_data_from_raw(&mut self, x: usize, y: usize, value : &str)  -> CedResult<()> {
        let key_column = self.get_column_if_valid(x, y)?;
        match key_column.column_type {
            ValueType::Text => self.set_data(x,y,Value::Text(value.to_string())),
            ValueType::Number => self.set_data(x,y,
                Value::Number(
                    value.parse().
                    map_err(|_| CedError::InvalidCellData(format!("Given value is \"{}\" which is not a number", value)))?
                )
            ),
        }
    }

    /// Set data by coordinate
    pub fn set_data(&mut self, x: usize, y: usize, value : Value) -> CedResult<()>  {
        let name = self.get_column_if_valid(x, y)?.name.to_owned();

        self.is_valid_column_data(y, &value)?;
        self.rows[x].update_value(&name, value);

        Ok(())
    }

    // THis should insert row with given column limiters
    pub fn insert_row(&mut self, row: usize, source : Option<&Vec<Value>>) -> CedResult<()> {
        let mut new_row = Row::new();
        if let Some(source) = source {
            self.check_row_length(source)?;
            self.columns.iter().zip(source.iter()).for_each(|(col,v)| new_row.insert_value(&col.name, v.clone()));
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
        if row_count == 0 || row_count < row { return None; }
        Some(self.rows.remove(row))
    }

    pub fn insert_column(&mut self, column : usize, column_name: &str, column_type: ValueType, limiter: Option<ValueLimiter>) {
        let new_column = Column::new(column_name,column_type, limiter);
        let default_value = new_column.get_default_value();
        for row in &mut self.rows {
            row.insert_value(&new_column.name, default_value.clone());
        }
        self.columns.insert(column, new_column);
    }

    pub fn delete_column(&mut self, column : usize) -> CedResult<()> {
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
    
    // <DRY>
    fn is_valid_cell_coordinate(&self, x:usize,y:usize) -> bool {
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

    // TODO
    fn is_valid_column_data(&self, column: usize,value: &Value) -> CedResult<()> {
        Ok(())
    }

    fn check_row_length(&self, values : &Vec<Value>) -> CedResult<()> {
        match self.get_column_count().cmp(&values.len()) {
            Ordering::Equal => (),
            Ordering::Less | Ordering::Greater => return Err(CedError::InvalidRowData(format!(r#"Given row length is "{}" while columns length is "{}""#, values.len(), self.get_column_count()))),
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
    // </EXT>
}

pub struct Column {
    pub(crate) name       : String,
    pub(crate) column_type: ValueType,
    pub(crate) limiter    : ValueLimiter,
}

impl Column {
    pub fn new(name: &str, column_type: ValueType, limiter: Option<ValueLimiter>) -> Self {
        Self {
            name: name.to_string(),
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
