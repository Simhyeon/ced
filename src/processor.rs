use crate::value::{ValueType, ValueLimiter};
use crate::virtual_data::VirtualData;
use crate::models::Direction;
use crate::error::CedResult;

pub struct Processor {
    cursor: Cursor,
    data  : VirtualData,
}

// TODO
// Possibly add row, column based edit function such as
// edit_row or edit_column
impl Processor {
    pub fn edit_cell(&mut self, x: usize, y:usize, input: &str) -> CedResult<()> {
        self.data.set_data_from_raw(x, y, input)?;
        Ok(())
    }

    // TODO
    pub fn clear_cell(&mut self,x: usize, y: usize) {

    }

    // TODO
    pub fn edit_row(&mut self) {

    }

    // TODO
    pub fn clear_row(&mut self) {

    }

    // TODO
    pub fn clear_column(&mut self) {

    }

    pub fn move_cursor(&mut self, direction: Direction) {
        self.cursor.move_by_direction(direction);
    }

    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.cursor.set(x, y);
    }

    pub fn add_row(&mut self, row_number: usize) {
        self.data.insert_row(row_number);
    }

    pub fn remove_row(&mut self, row_number: usize) {
        self.data.delete_row(row_number);
    }

    pub fn add_column(&mut self, column_number: usize, column_name: &str,column_type: ValueType, limiter: Option<ValueLimiter>) {
        self.data.insert_column(column_number, column_name, column_type, limiter);
    }

    pub fn remove_column(&mut self, column_number: usize) {
        self.data.delete_column(column_number);
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
