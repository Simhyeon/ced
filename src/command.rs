#[cfg(feature = "cli")]
use crate::cli::help;
use crate::error::{CedError, CedResult};
#[cfg(feature = "cli")]
use crate::page::Page;
use crate::processor::Processor;
use crate::utils::{self, subprocess};
use dcsv::{Column, Row, LIMITER_ATTRIBUTE_LEN, SCHEMA_HEADER};
use dcsv::{Value, ValueLimiter, ValueType};
use std::io::Write;
use std::str::FromStr;
use std::{ops::Sub, path::Path};
use utils::DEFAULT_DELIMITER;

/// Types of command
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CommandType {
    #[cfg(feature = "cli")]
    Version,
    #[cfg(feature = "cli")]
    Help,
    Undo,
    Redo,
    Create,
    Write,
    Import,
    ImportRaw,
    Export,
    AddRow,
    AddColumn,
    DeleteRow,
    DeleteColumn,
    EditCell,
    EditColumn,
    RenameColumn,
    EditRow,
    #[cfg(feature = "cli")]
    EditRowMultiple,
    MoveRow,
    MoveColumn,
    Exit,
    Execute,
    Print,
    PrintCell,
    PrintRow,
    PrintColumn,
    Limit,
    #[cfg(feature = "cli")]
    LimitPreset,
    Schema,
    SchemaInit,
    SchemaExport,
    History,
    None,
}

impl std::fmt::Display for CommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

// TODO
// Currently this never fails
impl FromStr for CommandType {
    type Err = CedError;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let command_type = match src.to_lowercase().trim() {
            #[cfg(feature = "cli")]
            "version" | "v" => Self::Version,
            #[cfg(feature = "cli")]
            "help" | "h" => Self::Help,
            "import" | "i" => Self::Import,
            "import-raw" | "ir" => Self::ImportRaw,
            "export" | "x" => Self::Export,
            "execute" | "ex" => Self::Execute,
            "create" | "c" => Self::Create,
            "write" | "w" => Self::Write,
            "print" | "p" => Self::Print,
            "print-cell" | "pc" => Self::PrintCell,
            "print-row" | "pr" => Self::PrintRow,
            "print-column" | "pl" => Self::PrintColumn,
            "add-row" | "ar" => Self::AddRow,
            "exit" | "quit" | "q" => Self::Exit,
            "add-column" | "ac" => Self::AddColumn,
            "delete-row" | "dr" => Self::DeleteRow,
            "delete-column" | "dc" => Self::DeleteColumn,
            "edit" | "edit-cell" | "e" => Self::EditCell,
            "edit-row" | "er" => Self::EditRow,
            #[cfg(feature = "cli")]
            "edit-row-multiple" | "erm" => Self::EditRowMultiple,
            "edit-column" | "ec" => Self::EditColumn,
            "rename-column" | "rc" => Self::RenameColumn,
            "move-row" | "move" | "m" => Self::MoveRow,
            "move-column" | "mc" => Self::MoveColumn,
            "limit" | "l" => Self::Limit,
            #[cfg(feature = "cli")]
            "limit-preset" | "lp" => Self::LimitPreset,
            "undo" | "u" => Self::Undo,
            "redo" | "r" => Self::Redo,
            "schema" | "s" => Self::Schema,
            "schema-init" | "si" => Self::SchemaInit,
            "schema-export" | "se" => Self::SchemaExport,
            "history" | "y" => Self::History,
            _ => {
                Self::None

                // TODO
                // Disabled error branch for current compatiblity
                //return Err(CedError::CommandError(format!(
                //"{} is not a valid command type",
                //src
                //)))
            }
        };
        Ok(command_type)
    }
}

/// Ergonomic wrapper around processor api
#[derive(Debug)]
pub struct Command {
    pub command_type: CommandType,
    pub arguments: Vec<String>,
}

impl Default for Command {
    fn default() -> Self {
        Self {
            command_type: CommandType::Print,
            arguments: vec![],
        }
    }
}

impl FromStr for Command {
    type Err = CedError;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let src: Vec<String> = utils::tokens_with_quote(src);
        let command = &src[0];
        let command_type = CommandType::from_str(command)?;
        Ok(Self {
            command_type,
            arguments: src[1..].iter().map(|s| s.to_string()).collect(),
        })
    }
}

/// Affected data container
#[allow(dead_code)]
enum CommandResult {
    Cell(usize, usize, Value),
    Rows(Vec<Row>),
    Columns(Vec<Column>),
}

// Default history capacity is a double word
#[cfg(feature = "cli")]
const HISTORY_CAPACITY: usize = 16;
#[cfg(feature = "cli")]
pub struct CommandHistory {
    pub index: usize,
    newest_snapshot: Option<HistoryRecord>,
    pub(crate) memento_history: Vec<HistoryRecord>,
    history_capacity: usize,
}

#[cfg(feature = "cli")]
impl CommandHistory {
    pub fn new() -> Self {
        let capacity = if let Ok(cap) = std::env::var("CED_HISTORY_CAPACITY") {
            if let Ok(num) = cap.parse::<usize>() {
                num
            } else {
                HISTORY_CAPACITY
            }
        } else {
            HISTORY_CAPACITY
        };
        Self {
            index: 0, // 0 should mean nothing rather than "first" element
            newest_snapshot: None,
            memento_history: vec![],
            history_capacity: capacity,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.memento_history.is_empty()
    }

    // If index is equal to lenth than there is no undo operation took in place
    pub(crate) fn is_newest(&self) -> bool {
        self.index >= self.memento_history.len()
    }

    pub(crate) fn set_current_backup(&mut self, data: Page) {
        self.newest_snapshot
            .replace(HistoryRecord::new(data, CommandType::Undo));
    }

    pub(crate) fn take_snapshot(&mut self, data: &Page, command: CommandType) {
        // Remove discarded changes
        // User will lose all undo history after current index if user undid several steps and had
        // done a new action
        self.drain_history();

        self.memento_history
            .push(HistoryRecord::new(data.clone(), command));
        // You cannot redo if you have done something other than undo
        if self.memento_history.len() > self.history_capacity {
            self.memento_history.rotate_left(1);
            self.memento_history.pop();
        } else {
            // Increase index according to history size increase
            self.index += 1;
        }
    }

    fn drain_history(&mut self) {
        if !self.memento_history.is_empty() && self.index < self.memento_history.len() {
            self.memento_history.drain(self.index..);
            self.newest_snapshot.take();
        }
    }

    pub(crate) fn get_undo(&mut self) -> Option<&HistoryRecord> {
        // Cannot go backward because index is 0
        if self.index == 0 {
            None
        } else {
            self.index -= 1;
            let target_index = self.index;
            self.memento_history.get(target_index)
        }
    }

    pub(crate) fn get_redo(&mut self) -> Option<&HistoryRecord> {
        match self.index {
            x if x == self.memento_history.len() => None,
            y if y == self.memento_history.len() - 1 => {
                self.index += 1;
                self.newest_snapshot.as_ref()
            }
            _ => {
                self.index += 1;
                let target_index = self.index;
                self.memento_history.get(target_index)
            }
        }
    }
}

#[cfg(feature = "cli")]
pub(crate) struct HistoryRecord {
    pub(crate) data: Page,
    pub(crate) command: CommandType,
}

#[cfg(feature = "cli")]
impl HistoryRecord {
    pub fn new(data: Page, command: CommandType) -> Self {
        Self { data, command }
    }
}

/// Main loop struct for interactive csv editing
impl Processor {
    /// Execute given command
    pub fn execute_command(&mut self, command: &Command) -> CedResult<()> {
        let page_name = &self
            .get_cursor()
            .ok_or_else(|| CedError::CommandError("Current page is empty".to_string()))?;
        match &command.command_type {
            #[cfg(feature = "cli")]
            CommandType::Version => help::print_version(),
            #[cfg(feature = "cli")]
            CommandType::Help => self.print_help_from_args(&command.arguments)?,
            CommandType::None => return Err(CedError::CommandError("No such command".to_string())),
            CommandType::Import => {
                #[cfg(feature = "cli")]
                self.drop_pages()?;
                self.import_file_from_args(&command.arguments, false)?
            }
            CommandType::ImportRaw => self.import_file_from_args(&command.arguments, true)?,
            CommandType::Schema => self.import_schema_from_args(page_name, &command.arguments)?,
            CommandType::SchemaInit => self.init_schema_from_args(&command.arguments)?,
            CommandType::SchemaExport => {
                self.export_schema_from_args(page_name, &command.arguments)?
            }
            CommandType::Export => self.write_to_file_from_args(page_name, &command.arguments)?,
            CommandType::Write => {
                self.overwrite_to_file_from_args(page_name, &command.arguments)?
            }
            CommandType::Create => {
                self.add_column_array(page_name, &command.arguments)?;
                self.log("New columns added\n")?;
            }
            CommandType::Print => self.print(page_name, &command.arguments)?,
            CommandType::PrintCell => self.print_cell(page_name, &command.arguments)?,
            CommandType::PrintRow => self.print_row(page_name, &command.arguments)?,
            CommandType::PrintColumn => self.print_column(page_name, &command.arguments)?,
            CommandType::AddRow => self.add_row_from_args(page_name, &command.arguments)?,
            CommandType::DeleteRow => self.remove_row_from_args(page_name, &command.arguments)?,
            CommandType::DeleteColumn => {
                self.remove_column_from_args(page_name, &command.arguments)?
            }
            CommandType::AddColumn => self.add_column_from_args(page_name, &command.arguments)?,
            CommandType::EditCell => self.edit_cell_from_args(page_name, &command.arguments)?,
            CommandType::EditRow => self.edit_row_from_args(page_name, &command.arguments)?,
            #[cfg(feature = "cli")]
            CommandType::EditRowMultiple => {
                self.edit_rows_from_args(page_name, &command.arguments)?
            }
            CommandType::EditColumn => self.edit_column_from_args(page_name, &command.arguments)?,
            CommandType::RenameColumn => {
                self.rename_column_from_args(page_name, &command.arguments)?
            }
            CommandType::MoveRow => self.move_row_from_args(page_name, &command.arguments)?,
            CommandType::MoveColumn => self.move_column_from_args(page_name, &command.arguments)?,
            CommandType::Limit => self.limit_column_from_args(page_name, &command.arguments)?,
            #[cfg(feature = "cli")]
            CommandType::LimitPreset => self.limit_preset(page_name, &command.arguments)?,
            CommandType::Execute => self.execute_from_file(&command.arguments)?,

            // NOTE
            // This is not handled by processor in current implementation
            CommandType::Exit | CommandType::Undo | CommandType::Redo | CommandType::History => (),
        }
        Ok(())
    }

    fn move_row_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CommandError(
                "Insufficient arguments for move-row".to_string(),
            ));
        }
        let src_number = args[0].parse::<usize>().map_err(|_| {
            CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
        })?;
        let target_number = args[1].parse::<usize>().map_err(|_| {
            CedError::CommandError(format!("\"{}\" is not a valid row number", args[1]))
        })?;
        self.move_row(page_name, src_number, target_number)?;
        self.log(&format!(
            "Row moved from \"{}\" to \"{}\"\n",
            src_number, target_number
        ))?;
        Ok(())
    }

    fn move_column_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CommandError(
                "Insufficient arguments for move-column".to_string(),
            ));
        }
        let src_number = self
            .get_page_data(page_name)?
            .try_get_column_index(&args[0])
            .ok_or_else(|| {
                CedError::InvalidColumn(format!("Column : \"{}\" is not valid", args[0]))
            })?;
        let target_number = self
            .get_page_data(page_name)?
            .try_get_column_index(&args[1])
            .ok_or_else(|| {
                CedError::InvalidColumn(format!("Column : \"{}\" is not valid", args[1]))
            })?;
        self.move_column(page_name, src_number, target_number)?;
        self.log(&format!(
            "Column moved from \"{}\" to \"{}\"\n",
            src_number, target_number
        ))?;
        Ok(())
    }

    fn rename_column_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CommandError(
                "Insufficient arguments for rename-column".to_string(),
            ));
        }

        let column = &args[0];
        let new_name = &args[1];

        self.rename_column(page_name, column, new_name)?;
        self.log(&format!(
            "Column renamed from \"{}\" to \"{}\"\n",
            column, new_name
        ))?;
        Ok(())
    }

    /// Edit single row
    fn edit_row_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        let len = args.len();
        let row_number: usize;
        match len {
            0 => {
                // No row
                return Err(CedError::CommandError(
                    "Insufficient arguments for edit-row".to_string(),
                ));
            }
            #[cfg(feature = "cli")]
            1 => {
                self.check_no_loop()?;
                // Only row
                row_number = args[0].parse::<usize>().map_err(|_| {
                    CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                utils::write_to_stdout("Type comma(,) to exit input\n")?;
                let values = self.edit_row_loop(page_name, Some(row_number))?;
                if values.is_empty() {
                    return Ok(());
                }
                self.edit_row(page_name, row_number, &values)?;
            }
            _ => {
                // From 2.. row + data
                row_number = args[0].parse::<usize>().map_err(|_| {
                    CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                let values = args[1]
                    .split(DEFAULT_DELIMITER)
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                self.set_row_from_string_array(page_name, row_number, &values)?;
            }
        }

        self.log(&format!("Row \"{}\" 's content changed\n", row_number))?;
        Ok(())
    }

    /// Edit multiple rows
    #[cfg(feature = "cli")]
    fn edit_rows_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        self.check_no_loop()?;
        let len = args.len();
        let mut start_index = 0;
        let mut end_index = self.get_row_count(page_name)?.max(1) - 1;
        match len {
            // No row
            0 => {}
            1 => {
                // Only starting row
                start_index = args[0].parse::<usize>().map_err(|_| {
                    CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
            }
            _ => {
                // From 2.. Starting row + ending row
                start_index = args[0].parse::<usize>().map_err(|_| {
                    CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                end_index = args[1].parse::<usize>().map_err(|_| {
                    CedError::CommandError(format!("\"{}\" is not a valid row number", args[1]))
                })?;
            }
        }

        utils::write_to_stdout("Type comma(,) to exit input\n")?;
        let mut edit_target_values = vec![];
        // Inclusive range
        for index in start_index..=end_index {
            let values = self.edit_row_loop(page_name, Some(index))?;
            if values.is_empty() {
                return Ok(());
            }
            edit_target_values.push((index, values));
        }

        // Edit rows only if every operation was succesfull
        for (index, values) in edit_target_values {
            self.edit_row(page_name, index, &values)?;
        }

        self.log(&format!(
            "Rows {}~{} contents changed\n",
            start_index, end_index
        ))?;
        Ok(())
    }

    fn edit_column_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CommandError(
                "Insufficient arguments for edit-column".to_string(),
            ));
        }

        let column = &args[0];
        let new_value = &args[1];

        self.edit_column(page_name, column, new_value)?;
        self.log(&format!(
            "Column \"{}\" content changed to \"{}\"\n",
            column, new_value
        ))?;
        Ok(())
    }

    fn edit_cell_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        if args.is_empty() {
            return Err(CedError::CommandError("Edit needs coordinate".to_string()));
        }

        let coord = &args[0].split(',').collect::<Vec<&str>>();
        let value = if args.len() >= 2 {
            args[1..].join(" ")
        } else {
            String::new()
        };
        if coord.len() != 2 {
            return Err(CedError::CommandError(
                "Cell cooridnate should be in a form of \"row,column\"".to_string(),
            ));
        }

        if !utils::is_valid_csv(&value) {
            return Err(CedError::CommandError(
                "Given cell value is not a valid csv value".to_string(),
            ));
        }

        let row = coord[0].parse::<usize>().map_err(|_| {
            CedError::CommandError(format!("\"{}\" is not a valid row number", coord[0]))
        })?;
        let column = self
            .get_page_data(page_name)?
            .try_get_column_index(coord[1])
            .ok_or_else(|| {
                CedError::InvalidColumn(format!("Column : \"{}\" is not valid", coord[1]))
            })?;

        self.edit_cell(page_name, row, column, &value)?;
        self.log(&format!(
            "Cell \"({},{})\" content changed to \"{}\"\n",
            row, coord[1], &&value
        ))?;
        Ok(())
    }

    fn add_row_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        let len = args.len();
        let row_number: usize;
        match len {
            #[cfg(feature = "cli")]
            0 => {
                self.check_no_loop()?;
                // No row number
                row_number = self.get_row_count(page_name)?;
                if row_number > self.get_row_count(page_name)? {
                    return Err(CedError::InvalidColumn(format!(
                        "Cannot add row to out of range position : {}",
                        row_number
                    )));
                }
                utils::write_to_stdout("Type comma(,) to exit input\n")?;
                let values = self.add_row_loop(page_name, None)?;
                if values.is_empty() {
                    return Ok(());
                }
                self.add_row(page_name, row_number, Some(&values))?;
            }
            #[cfg(feature = "cli")]
            1 => {
                self.check_no_loop()?;
                // Only row number
                row_number = args[0].parse::<usize>().map_err(|_| {
                    CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                if row_number > self.get_row_count(page_name)? {
                    return Err(CedError::InvalidColumn(format!(
                        "Cannot add row to out of range position : {}",
                        row_number
                    )));
                }
                utils::write_to_stdout("Type comma(,) to exit input\n")?;
                let values = self.add_row_loop(page_name, None)?;
                if values.is_empty() {
                    return Ok(());
                }
                self.add_row(page_name, row_number, Some(&values))?;
            }
            _ => {
                // From 2.. row + data
                row_number = args[0].parse::<usize>().map_err(|_| {
                    CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                let values = args[1]
                    .split(DEFAULT_DELIMITER)
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                self.add_row_from_string_array(page_name, row_number, &values)?;
            }
        }
        self.log(&format!("New row added to \"{}\"\n", row_number))?;
        Ok(())
    }

    // DRY code for value check
    /// Check value on loop variants
    ///
    /// Return value decided whether early return or not
    #[cfg(feature = "cli")]
    fn loop_value_check(value: &mut Value, type_mismatch: &mut bool) -> bool {
        // Check value content
        // Early return
        if let Value::Text(content) = value {
            if content == DEFAULT_DELIMITER {
                return true;
            }

            // Check csv validity
            if !utils::is_valid_csv(content) {
                // It is considered as type_mismatch
                *type_mismatch = true;
            }
        }
        false
    }

    #[cfg(feature = "cli")]
    fn check_no_loop(&self) -> CedResult<()> {
        if self.no_loop {
            return Err(CedError::CommandError(
                "Interactive loop is restricted. Breaking...".to_owned(),
            ));
        }
        Ok(())
    }

    #[cfg(feature = "cli")]
    fn add_row_loop(
        &mut self,
        page_name: &str,
        row_number: Option<usize>,
    ) -> CedResult<Vec<Value>> {
        let mut values = vec![];
        let columns = &self.get_page_data(page_name)?.get_columns();
        if columns.is_empty() {
            utils::write_to_stdout(": Csv is empty : \n")?;
            return Ok(vec![]);
        }
        for (idx, col) in columns.iter().enumerate() {
            let mut type_mismatch = false;
            let default = if let Some(row_number) = row_number {
                self.get_cell(page_name, row_number, idx)?
                    .ok_or(CedError::OutOfRangeError)?
                    .to_owned()
            } else {
                col.get_default_value()
            };
            utils::write_to_stdout(&format!("{}~{{{}}} = ", col.name, default))?;
            let value_src = utils::read_stdin(true)?;
            let mut value = if !value_src.is_empty() {
                match Value::from_str(&value_src, col.column_type) {
                    Ok(value) => value,
                    Err(_) => {
                        type_mismatch = true;
                        Value::Text(value_src)
                    }
                }
            } else {
                default.clone()
            };

            if Self::loop_value_check(&mut value, &mut type_mismatch) {
                utils::write_to_stdout(": Prompt interrupted :\n")?;
                return Ok(vec![]);
            }

            while !col.limiter.qualify(&value) || type_mismatch {
                type_mismatch = false;
                utils::write_to_stdout(
                    "Given value doesn't qualify column limiter or is not a valid csv value\n",
                )?;
                utils::write_to_stdout(&format!("{}~{{{}}} = ", col.name, default))?;
                let value_src = utils::read_stdin(true)?;
                value = if !value_src.is_empty() {
                    match Value::from_str(&value_src, col.column_type) {
                        Ok(value) => value,
                        Err(_) => {
                            type_mismatch = true;
                            Value::Text(value_src)
                        }
                    }
                } else {
                    default.clone()
                };

                if Self::loop_value_check(&mut value, &mut type_mismatch) {
                    utils::write_to_stdout(": Prompt interrupted :\n")?;
                    return Ok(vec![]);
                }
            }

            values.push(value);
        }
        Ok(values)
    }

    // None means value should not change
    #[cfg(feature = "cli")]
    fn edit_row_loop(
        &mut self,
        page_name: &str,
        row_number: Option<usize>,
    ) -> CedResult<Vec<Option<Value>>> {
        let mut values = vec![];
        let columns = &self.get_page_data(page_name)?.get_columns();
        if columns.is_empty() {
            utils::write_to_stdout(": Csv is empty : \n")?;
            return Ok(vec![]);
        }
        for (idx, col) in columns.iter().enumerate() {
            let mut type_mismatch = false;
            let default = if let Some(row_number) = row_number {
                self.get_cell(page_name, row_number, idx)?
                    .ok_or(CedError::OutOfRangeError)?
                    .to_owned()
            } else {
                col.get_default_value()
            };
            utils::write_to_stdout(&format!("{}~{{{}}} = ", col.name, default))?;
            let value_src = utils::read_stdin(true)?;
            let mut value = if !value_src.is_empty() {
                Some(match Value::from_str(&value_src, col.column_type) {
                    Ok(value) => value,
                    Err(_) => {
                        type_mismatch = true;
                        Value::Text(value_src)
                    }
                })
            } else {
                None
            };

            // Early return
            if let Some(value) = value.as_mut() {
                if Self::loop_value_check(value, &mut type_mismatch) {
                    utils::write_to_stdout(": Prompt interrupted :\n")?;
                    return Ok(vec![]);
                }
            }

            while value != None {
                // when value was not a "Not changed"
                if col.limiter.qualify(value.as_ref().unwrap()) && !type_mismatch {
                    break;
                }
                type_mismatch = false;
                utils::write_to_stdout("Given value doesn't qualify column limiter\n")?;
                utils::write_to_stdout(&format!("{}~{{{}}} = ", col.name, default))?;
                let value_src = utils::read_stdin(true)?;
                value = if !value_src.is_empty() {
                    Some(match Value::from_str(&value_src, col.column_type) {
                        Ok(value) => value,
                        Err(_) => {
                            type_mismatch = true;
                            Value::Text(value_src)
                        }
                    })
                } else {
                    None
                };

                // Early return
                if let Some(value) = value.as_mut() {
                    if Self::loop_value_check(value, &mut type_mismatch) {
                        utils::write_to_stdout(": Prompt interrupted :\n")?;
                        return Ok(vec![]);
                    }
                }
            }

            values.push(value);
        }
        Ok(values)
    }

    fn add_column_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        let mut column_number = self.get_page_data(page_name)?.get_column_count();
        let mut column_type = ValueType::Text;
        let mut placeholder = None;

        if args.is_empty() {
            return Err(CedError::CommandError(
                "Cannot add column without name".to_owned(),
            ));
        }

        let column_name = args[0].as_str();

        if args.len() >= 2 {
            column_number = args[1].parse::<usize>().map_err(|_| {
                CedError::CommandError(format!("\"{}\" is not a valid column number", args[1]))
            })?;
        }
        if args.len() >= 3 {
            column_type = ValueType::from_str(&args[2])?;
        }

        if args.len() >= 4 {
            placeholder.replace(Value::from_str(&args[3], column_type)?);
        }

        self.add_column(
            page_name,
            column_number,
            column_name,
            column_type,
            None,
            placeholder,
        )?;
        self.log(&format!(
            "New column \"{}\" added to \"{}\"\n",
            column_name, column_number
        ))?;
        Ok(())
    }

    fn remove_row_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        let row_count = if args.is_empty() {
            self.get_row_count(page_name)?
        } else {
            args[0].parse::<usize>().map_err(|_| {
                CedError::CommandError(format!("\"{}\" is not a valid index", args[0]))
            })?
        }
        .sub(1);

        if !self.remove_row(page_name, row_count)? {
            utils::write_to_stdout("No such row to remove\n")?;
        }
        self.log(&format!("A row removed from \"{}\"\n", row_count))?;
        Ok(())
    }

    fn remove_column_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        let column_count = if args.is_empty() {
            self.get_page_data(page_name)?.get_column_count()
        } else {
            self.get_page_data(page_name)?
                .try_get_column_index(&args[0])
                .ok_or_else(|| {
                    CedError::InvalidColumn(format!(
                        "Cannot remove non-existent column \"{}\"",
                        args[0]
                    ))
                })?
        };

        self.remove_column(page_name, column_count)?;
        self.log(&format!("A column \"{}\" removed\n", &args[0]))?;
        Ok(())
    }

    #[cfg(feature = "cli")]
    fn print_help_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.is_empty() {
            help::print_help_text();
        } else {
            help::print_command_help(Command::from_str(&args[0])?.command_type);
        }
        Ok(())
    }

    /// Read from args
    ///
    /// file is asssumed to have header
    /// You can give has_header value as second parameter
    fn import_file_from_args(&mut self, args: &Vec<String>, raw_mode: bool) -> CedResult<()> {
        match args.len() {
            0 => {
                return Err(CedError::CommandError(
                    "You have to specify a file name to import from".to_owned(),
                ))
            }
            1 => self.import_from_file(Path::new(&args[0]), true, None, raw_mode)?,
            _ => {
                // Optional line ending configuration
                let mut line_ending = None;
                if args.len() > 2 && &args[2].to_lowercase() == "cr" {
                    line_ending = Some('\r');
                }

                // Remove entry if already exists
                let page_name = &args[0];
                self.remove_page(page_name);

                self.import_from_file(
                    Path::new(page_name),
                    args[1].parse().map_err(|_| {
                        CedError::CommandError(format!(
                            "Given value \"{}\" should be a valid boolean value. ( has_header )",
                            args[1]
                        ))
                    })?,
                    line_ending,
                    raw_mode,
                )?
            }
        }
        let footer = if raw_mode { " as array mode" } else { "" };
        self.log(&format!("File \"{}\" imported{}\n", &args[0], footer))?;
        Ok(())
    }

    fn import_schema_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        if self.get_page_data(page_name)?.is_array() {
            return Err(CedError::InvalidPageOperation(
                "Cannot import schema in array mode".to_string(),
            ));
        }

        if args.len() < 2 {
            return Err(CedError::CommandError(
                "Insufficient variable for schema".to_owned(),
            ));
        }
        let schema_file = &args[0];
        let force = &args[1];
        self.set_schema(
            page_name,
            schema_file,
            !force
                .parse::<bool>()
                .map_err(|_| CedError::CommandError(format!("{} is not a valid value", force)))?,
        )?;
        self.log(&format!("Schema \"{}\" applied\n", &args[0]))?;
        Ok(())
    }

    fn export_schema_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        if self.get_page_data(page_name)?.is_array() {
            return Err(CedError::InvalidPageOperation(
                "Cannot export schema in array mode".to_string(),
            ));
        }

        if args.is_empty() {
            return Err(CedError::CommandError(
                "Schema export needs a file path".to_owned(),
            ));
        }
        let file = &args[0];
        let mut file = std::fs::File::create(file)
            .map_err(|err| CedError::io_error(err, "Failed to open file for schema write"))?;

        file.write_all(self.export_schema(page_name)?.as_bytes())
            .map_err(|err| CedError::io_error(err, "Failed to write schema to a file"))?;

        self.log(&format!("Schema exported to \"{}\"\n", args[0]))?;
        Ok(())
    }

    fn init_schema_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let file_name = if args.is_empty() {
            "ced_schema.csv"
        } else {
            &args[0]
        };
        let mut file = std::fs::File::create(file_name)
            .map_err(|err| CedError::io_error(err, "Failed to open file for schema init"))?;

        file.write_all(SCHEMA_HEADER.as_bytes())
            .map_err(|err| CedError::io_error(err, "Failed to write schema to a file"))?;

        self.log(&format!("Schema initiated to \"{}\"\n", file_name))?;
        Ok(())
    }

    fn write_to_file_from_args(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        if args.is_empty() {
            return Err(CedError::CommandError(
                "Export requires file path".to_owned(),
            ));
        }
        self.write_to_file(page_name, &args[0])?;
        self.log(&format!("File exported to \"{}\"\n", &args[0]))?;
        Ok(())
    }

    fn overwrite_to_file_from_args(
        &mut self,
        page_name: &str,
        args: &Vec<String>,
    ) -> CedResult<()> {
        let cache: bool = if !args.is_empty() {
            args[0].parse::<bool>().map_err(|_| {
                CedError::CommandError(format!("\"{}\" is not a valid boolean value", args[0]))
            })?
        } else {
            true
        };
        let success = self.overwrite_to_file(page_name, cache)?;
        if success {
            self.log("File overwritten successfully\n")?;
        } else {
            self.log(": No source file to write. Use export instead :\n")?;
        }
        Ok(())
    }

    // Combine ths with viewer variant
    fn print(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        let mut viewer = vec![];
        // Use given command
        // or use environment variable
        // External command has higher priority
        if !args.is_empty() {
            viewer = args[0..].to_vec();
        } else if let Ok(var) = std::env::var("CED_VIEWER") {
            viewer = utils::tokens_with_quote(&var);
        }

        if viewer.is_empty() {
            self.print_virtual_container(page_name)?;
        } else {
            let csv = self.get_page_as_string(page_name)?;
            self.print_with_viewer(csv, &viewer)?;
        }

        Ok(())
    }

    fn print_cell(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        if args.is_empty() {
            return Err(CedError::CommandError(
                "Cannot print cell without a cooridnate".to_owned(),
            ));
        }
        let mut print_mode = "simple";
        let coord = args[0].split(',').collect::<Vec<&str>>();
        if coord.len() < 2 {
            return Err(CedError::CommandError(format!(
                "\"{}\' is not a valid cell coordinate",
                args[0]
            )));
        }
        let (x, y) = (
            coord[0].parse::<usize>().map_err(|_| {
                CedError::CommandError("You need to feed usize number for coordinate".to_string())
            })?,
            self.get_page_data(page_name)?
                .try_get_column_index(coord[1])
                .ok_or_else(|| {
                    CedError::CommandError(
                        "You need to appropriate column for coordinate".to_string(),
                    )
                })?,
        );

        if args.len() >= 2 {
            print_mode = &args[1];
        }

        match self.get_page_data(page_name)?.get_cell(x, y) {
            Some(cell) => match print_mode.to_lowercase().as_str() {
                "v" | "verbose" => utils::write_to_stdout(&format!("{:?}\n", cell))?,
                "d" | "debug" => {
                    let col = self
                        .get_column(page_name, y)?
                        .ok_or(CedError::OutOfRangeError)?;
                    utils::write_to_stdout(&format!("{:#?}\n", col))?;
                    utils::write_to_stdout(&format!("Cell data : {:#?}\n", cell))?
                }
                _ => utils::write_to_stdout(&format!("{}\n", cell))?,
            },
            None => utils::write_to_stdout("No such cell\n")?,
        }

        Ok(())
    }

    fn print_row(&self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        if args.is_empty() {
            return Err(CedError::CommandError(
                "Print-row needs row number".to_string(),
            ));
        }
        let row_index = args[0].parse::<usize>().map_err(|_| {
            CedError::CommandError("You need to feed a valid number as a row number".to_string())
        })?;

        let row = self
            .get_page_data(page_name)?
            .get_row_as_string(row_index)?
            + "\n";

        let viewer: Vec<_>;
        // Use given command
        // or use environment variable
        // External command has higher priority
        if args.len() >= 2 {
            viewer = args[1..].to_vec();
            subprocess(&viewer, Some(row))
        } else if let Ok(var) = std::env::var("CED_VIEWER") {
            viewer = var.split_whitespace().map(|s| s.to_string()).collect();
            subprocess(&viewer, Some(row))
        } else {
            self.print_virtual_data_row(page_name, row_index, false)
        }
    }

    fn print_column(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        let mut print_mode = "simple";
        if args.is_empty() {
            let columns = self
                .get_page_data(page_name)?
                .get_columns()
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>()
                .join(",");
            utils::write_to_stdout(&format!(": --{}-- :\n", columns))?;
            return Ok(());
        }

        if args.len() >= 2 {
            print_mode = &args[1];
        }

        if let Some(col) = self
            .get_page_data(page_name)?
            .try_get_column_index(&args[0])
        {
            if col < self.get_page_data(page_name)?.get_column_count() {
                let col = self
                    .get_column(page_name, col)?
                    .ok_or(CedError::OutOfRangeError)?;
                match print_mode.to_lowercase().as_str() {
                    "debug" | "d" => utils::write_to_stdout(&format!("{:#?}\n", col))?,
                    "verbose" | "v" => utils::write_to_stdout(&format!(
                        "Column = \nName: {}\nType: {}\n---\nLimiter =\n{}",
                        col.get_name(),
                        col.get_column_type(),
                        col.limiter
                    ))?,
                    _ => utils::write_to_stdout(&format!(
                        "Name: {}\nType: {}\n",
                        col.get_name(),
                        col.get_column_type()
                    ))?,
                }
                return Ok(());
            }
        }
        utils::write_to_stdout("No such column\n")?;
        Ok(())
    }

    fn print_with_viewer(&self, csv: String, viewer: &[String]) -> CedResult<()> {
        subprocess(viewer, Some(csv))
    }

    fn print_virtual_data_row(
        &self,
        page_name: &str,
        row_index: usize,
        include_header: bool,
    ) -> CedResult<()> {
        let page = self.get_page_data(page_name)?;
        // Empty csv value, return early
        if page.get_row_count() == 0 {
            utils::write_to_stdout(": CSV is empty :\n")?;
            return Ok(());
        }

        if page.get_row_count() <= row_index {
            utils::write_to_stdout(": Given row index is not available :")?;
            return Ok(());
        }

        let digits_count = page.get_row_count().to_string().len();
        if include_header {
            // 0 length csv is panicking error at this moment, thus safe to unwrap
            let header_with_number = format!(
                "{: ^digits_count$}| {}\n",
                "H ",
                page.get_columns()
                    .iter()
                    .enumerate()
                    .map(|(i, col)| format!("[{}]:{}", i, col.name))
                    .collect::<Vec<String>>()
                    .join("")
            );
            utils::write_to_stdout(&header_with_number)?;
        }

        let row_string = page.get_row_as_string(row_index)?;
        utils::write_to_stdout(&format!("{: ^digits_count$} | {}\n", row_index, row_string))?;

        Ok(())
    }

    /// Print virtual container to console
    fn print_virtual_container(&self, page_name: &str) -> CedResult<()> {
        let page = self.get_page_data(page_name)?;
        // Empty csv value, return early
        if page.get_row_count() == 0 {
            utils::write_to_stdout(": CSV is empty :\n")?;
            return Ok(());
        }

        if page.is_array() {
            utils::write_to_stdout("-- Mode: Array --\n")?;
        }
        let digits_count = page.get_row_count().to_string().len();
        // 0 length csv is panicking error at this moment, thus safe to unwrap
        let header_with_number = format!(
            "{: <digits_count$} | {}\n",
            "H",
            page.get_columns()
                .iter()
                .enumerate()
                .map(|(i, col)| format!("[{}]:{}", i, col.name))
                .collect::<Vec<String>>()
                .join("")
        );
        utils::write_to_stdout(&header_with_number)?;

        let rows = self.get_page_data(page_name)?.get_rows();
        for (index, row) in rows.iter().enumerate() {
            let row_string = row
                .iter()
                .enumerate()
                .map(|(i, cell)| format!("[{}]:{}", i, cell))
                .collect::<Vec<_>>()
                .join("");
            utils::write_to_stdout(&format!("{: <digits_count$} | {}\n", index, row_string))?;
        }

        Ok(())
    }

    pub fn limit_column_from_args(&mut self, page_name: &str, args: &[String]) -> CedResult<()> {
        if self.get_page_data(page_name)?.is_array() {
            return Err(CedError::InvalidPageOperation(
                "Cannot set limiter in array mode".to_string(),
            ));
        }

        if args.is_empty() {
            self.add_limiter_prompt(page_name)?;
        } else {
            let args = if args.len() == 1 {
                args[0].split(',').collect::<Vec<_>>()
            } else {
                return Err(CedError::CommandError(
                    "Incorrect arguments for limit".to_string(),
                ));
            };
            if args.len() != LIMITER_ATTRIBUTE_LEN + 2 {
                println!("{:#?}", args);
                return Err(CedError::CommandError(
                    "Incorrect arguments for limit, needs 6 values".to_string(),
                ));
            }

            let column_name = args.first().unwrap();
            let force_update = args.last().unwrap();

            let force = if force_update.is_empty() {
                true
            } else {
                force_update.parse::<bool>().map_err(|_| {
                    CedError::CommandError(
                        "You need to feed boolean value for the force value".to_string(),
                    )
                })?
            };

            let limiter = ValueLimiter::from_line(&args[1..=LIMITER_ATTRIBUTE_LEN])?;

            self.set_limiter(page_name, column_name, &limiter, !force)?;
            self.log(&format!("Limited column \"{}\"\n", column_name))?;
        }
        Ok(())
    }

    #[cfg(feature = "cli")]
    pub fn limit_preset(&mut self, page_name: &str, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CommandError(
                "Limit-preset needs column and preset_name".to_owned(),
            ));
        }
        let column = &args[0];
        let preset_name = &args[1];
        let mut panic = true;
        if args.len() >= 3 {
            panic = !args[2].parse::<bool>().map_err(|_| {
                CedError::CommandError(
                    "You need to feed boolean value for the force value".to_string(),
                )
            })?;
        }
        self.set_limiter_from_preset(page_name, column, preset_name, panic)?;
        Ok(())
    }

    pub fn execute_from_file(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.is_empty() {
            return Err(CedError::CommandError(
                "Execute needs a file to read from".to_string(),
            ));
        }
        let file = &args[0];
        let content = std::fs::read_to_string(file).map_err(|err| {
            CedError::io_error(
                err,
                &format!("Failed to read file \"{}\" for execution", file),
            )
        })?;
        // Split by line
        for (idx, line) in content.lines().enumerate() {
            // Split by semi colon
            for comm in line.split_terminator(';') {
                if let Err(err) = self.execute_command(&Command::from_str(comm)?) {
                    utils::write_to_stderr(&format!(
                        "Line : {} -> Failed to execute command : \"{}\"\n",
                        idx + 1,
                        comm
                    ))?;
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    fn add_limiter_prompt(&mut self, page_name: &str) -> CedResult<()> {
        let limiter_prompts = vec![
            "Column = ",
            "Type (Text|Number) = ",
            "Default = ",
            "Variants(a b c) = ",
            "Pattern = ",
            "Force update(default=true) = ",
        ];
        let mut limiter_attributes = vec![];

        // Print columns before limiter prompt
        let page = self.pages.get(page_name);
        match page {
            Some(page) => {
                let columns = page.get_columns();
                if columns.is_empty() {
                    utils::write_to_stdout(": Csv is empty : \n")?;
                    return Ok(());
                }

                let columns = columns
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                utils::write_to_stdout(&format!(": --{}-- :\n", columns))?;
            }
            None => {
                utils::write_to_stdout(": Csv is empty : \n")?;
                return Ok(());
            }
        }

        for prompt in limiter_prompts {
            utils::write_to_stdout(prompt)?;
            let input = utils::read_stdin(true)?;

            // Interrupt
            if input == DEFAULT_DELIMITER {
                return Ok(());
            }

            limiter_attributes.push(input);
        }

        // +2 is necessary because given input also includes target column & force update
        if limiter_attributes.len() != LIMITER_ATTRIBUTE_LEN + 2 {
            return Err(CedError::InvalidPageOperation(format!(
                "Limit needs \"{}\" arguments but given \"{}\"",
                LIMITER_ATTRIBUTE_LEN + 2,
                limiter_attributes.len()
            )));
        }

        let column_name = limiter_attributes.first().unwrap();
        let force_update = limiter_attributes.last().unwrap();

        let force = if force_update.is_empty() {
            true
        } else {
            force_update.parse::<bool>().map_err(|_| {
                CedError::CommandError(
                    "You need to feed boolean value for the force value".to_string(),
                )
            })?
        };

        let limiter = ValueLimiter::from_line(&limiter_attributes[1..=LIMITER_ATTRIBUTE_LEN])?;
        self.set_limiter(page_name, column_name, &limiter, !force)?;
        self.log(&format!("Limited column \"{}\"\n", column_name))?;
        Ok(())
    }
}
