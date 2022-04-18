use super::utils;
use crate::error::{CedError, CedResult};
use crate::processor::Processor;
use crate::value::ValueType;
use crate::virtual_data::{VirtualData, SCHEMA_HEADER, Row, Column};
use crate::{ValueLimiter, Value, help};
use std::io::Write;
use std::process::Stdio;
use std::{ops::Sub, path::Path};

#[derive(PartialEq, Debug)]
pub enum CommandType {
    Version,
    Help,
    Undo,
    Redo,
    Create,
    Write,
    Import,
    Export,
    AddRow,
    AddColumn,
    DeleteRow,
    DeleteColumn,
    EditCell,
    EditColumn,
    RenameColumn,
    EditRow,
    EditRowMultiple,
    MoveRow,
    MoveColumn,
    Exit,
    Print,
    PrintCell,
    PrintColumn,
    Limit,
    Schema,
    SchemaInit,
    SchemaExport,
    None,
}

impl CommandType {
    pub fn from_str(src: &str) -> Self {
        let command_type = match src.to_lowercase().trim() {
            "version" | "v" => Self::Version,
            "help" | "h" => Self::Help,
            "import" | "i" => Self::Import,
            "export" | "x" => Self::Export,
            "create" | "c" => Self::Create,
            "write" | "w" => Self::Write,
            "print" | "p" => Self::Print,
            "print-cell" | "pc" => Self::PrintCell,
            "print-column" | "pl" => Self::PrintColumn,
            "add-row" | "ar" => Self::AddRow,
            "exit" | "quit" | "q" => Self::Exit,
            "add-column" | "ac" => Self::AddColumn,
            "delete-row" | "dr" => Self::DeleteRow,
            "delete-column" | "dc" => Self::DeleteColumn,
            "edit" | "edit-cell" | "e" => Self::EditCell,
            "edit-row" | "er" => Self::EditRow,
            "edit-row-multiple" | "erm" => Self::EditRowMultiple,
            "edit-column" | "ec" => Self::EditColumn,
            "rename-column" | "rc" => Self::RenameColumn,
            "move-row" | "move" | "m" => Self::MoveRow,
            "move-column" | "mc" => Self::MoveColumn,
            "limit" | "l" => Self::Limit,
            "undo" | "u" => Self::Undo,
            "redo" | "r" => Self::Redo,
            "schema" | "s" => Self::Schema,
            "schema-init" | "si" => Self::SchemaInit,
            "schema-export" | "se" => Self::SchemaExport,
            _ => Self::None,
        };
        command_type
    }
}

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

impl Command {
    pub fn from_str(src: &str) -> CedResult<Self> {
        let src: Vec<&str> = src.split_whitespace().collect();
        let command = src[0];
        let command_type = CommandType::from_str(command);
        Ok(Self {
            command_type,
            arguments: src[1..].iter().map(|s| s.to_string()).collect(),
        })
    }
}

/// Affected data container
#[allow(dead_code)]
enum CommandResult {
    Cell(usize,usize,Value),
    Rows(Vec<Row>),
    Columns(Vec<Column>),
}

#[allow(dead_code)]
struct CommandRecord {
    command : Command,
    command_result : CommandResult,
}

const HISTORY_CAPACITY: usize = 16;
struct CommandHistory {
    // TODO
    // Preserved for command pattern
    // history: Vec<Command>,
    memento_history: Vec<VirtualData>,
    redo_history: Vec<VirtualData>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            memento_history: vec![],
            redo_history: vec![],
        }
    }

    pub(crate) fn take_snapshot(&mut self, data: &VirtualData) {
        self.memento_history.push(data.clone());
        // You cannot redo if you have done something other than undo
        self.redo_history.clear();
        if self.memento_history.len() > HISTORY_CAPACITY {
            self.memento_history.rotate_left(1);
            self.memento_history.pop();
        }
    }

    pub(crate) fn pop(&mut self) -> Option<&VirtualData> {
        if let Some(data) = self.memento_history.pop() {
            self.redo_history.push(data);
            self.redo_history.last()
        } else {
            None
        }
    }

    pub(crate) fn pull_redo(&mut self) -> Option<VirtualData> {
        self.redo_history.pop()
    }
}

/// Main loop struct for interactive csv editing
pub struct CommandLoop {
    history: CommandHistory,
    processor: Processor,
}

impl CommandLoop {
    pub fn new() -> Self {
        Self {
            history: CommandHistory::new(),
            processor: Processor::new(),
        }
    }

    pub fn feed_command(&mut self, command: &Command, panic: bool) -> CedResult<()> {
        self.execute_command(command, panic)?;
        Ok(())
    }

    /// Start a loop until exit
    pub fn start_loop(&mut self) -> CedResult<()> {
        let mut command = Command::default();
        utils::write_to_stdout("Ced, a csv editor\n")?;
        while CommandType::Exit != command.command_type {
            utils::write_to_stdout(">> ")?;
            let input = &utils::read_stdin(true)?;
            if input.is_empty() {
                continue;
            }
            command = Command::from_str(input)?;
            self.execute_command(&command, false)?;
        }
        Ok(())
    }

    /// This never fails
    fn execute_command(&mut self, command: &Command, panic: bool) -> CedResult<()> {
        // DEBUG NOTE TODO
        #[cfg(debug_assertions)]
        utils::write_to_stderr(&format!("{:?}\n", command))?;

        match command.command_type {
            CommandType::Undo | CommandType::Redo => {
                if command.command_type == CommandType::Undo {
                    self.undo()?;
                } else {
                    self.redo()?;
                }
                return Ok(());
            }
            // Un-redoable commands
            CommandType::Help
            | CommandType::Exit
            | CommandType::Import
            | CommandType::Export
            | CommandType::Create
            | CommandType::Write
            | CommandType::None
            | CommandType::Schema
            | CommandType::SchemaInit
            | CommandType::SchemaExport
            | CommandType::Version
            | CommandType::PrintCell
            | CommandType::PrintColumn 
            | CommandType::Print => (),
            _ => self.history.take_snapshot(&self.processor.data),
        }

        if let Err(err) = self.processor.execute_command(&command) {
            if panic {
                return Err(err);
            } else {
                utils::write_to_stderr(&(err.to_string() + "\n"))?;
            }
        }
        Ok(())
    }

    fn undo(&mut self) -> CedResult<()> {
        if let Some(prev) = self.history.pop() {
            self.processor.data = prev.clone();
        }
        Ok(())
    }

    fn redo(&mut self) -> CedResult<()> {
        if let Some(prev) = self.history.pull_redo() {
            self.processor.data = prev;
        }
        Ok(())
    }
}

impl Processor {
    /// Execute given command
    pub fn execute_command(&mut self, command: &Command) -> CedResult<()> {
        match command.command_type {
            CommandType::Version => help::print_version(),
            CommandType::None => utils::write_to_stdout("No such command \n")?,
            CommandType::Help => self.print_help_from_args(&command.arguments)?,
            CommandType::Import => self.import_file_from_args(&command.arguments)?,
            CommandType::Schema => self.import_schema_from_args(&command.arguments)?,
            CommandType::SchemaInit => self.init_schema_from_args(&command.arguments)?,
            CommandType::SchemaExport => self.export_schema_from_args(&command.arguments)?,
            CommandType::Export => self.write_to_file_from_args(&command.arguments)?,
            CommandType::Write => self.overwrite_to_file_from_args(&command.arguments)?,
            CommandType::Create => {
                self.add_column_array(&command.arguments)?;
                utils::write_to_stdout("New columns added\n")?;
            }
            CommandType::Print => self.print(&command.arguments)?,
            CommandType::PrintCell => self.print_cell(&command.arguments)?,
            CommandType::PrintColumn => self.print_column(&command.arguments)?,
            CommandType::AddRow => self.add_row_from_args(&command.arguments)?,
            CommandType::DeleteRow => self.remove_row_from_args(&command.arguments)?,
            CommandType::DeleteColumn => self.remove_column_from_args(&command.arguments)?,
            CommandType::AddColumn => self.add_column_from_args(&command.arguments)?,
            CommandType::EditCell => self.edit_cell_from_args(&command.arguments)?,
            CommandType::EditRow => self.edit_row_from_args(&command.arguments)?,
            CommandType::EditRowMultiple => self.edit_rows_from_args(&command.arguments)?,
            CommandType::EditColumn => self.edit_column_from_args(&command.arguments)?,
            CommandType::RenameColumn => self.rename_column_from_args(&command.arguments)?,
            CommandType::MoveRow => self.move_row_from_args(&command.arguments)?,
            CommandType::MoveColumn => self.move_column_from_args(&command.arguments)?,
            CommandType::Limit => self.limit_column_from_args(&command.arguments)?,
            _ => (),
        }
        Ok(())
    }

    fn move_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CliError(format!(
                "Insufficient arguments for move-row"
            )));
        }
        let src_number = args[0].parse::<usize>().map_err(|_| {
            CedError::CliError(format!("\"{}\" is not a valid row number", args[0]))
        })?;
        let target_number = args[1].parse::<usize>().map_err(|_| {
            CedError::CliError(format!("\"{}\" is not a valid row number", args[1]))
        })?;
        self.move_row(src_number, target_number)?;
        utils::write_to_stdout("Row moved\n")?;
        Ok(())
    }

    fn move_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CliError(format!(
                "Insufficient arguments for move-column"
            )));
        }
        let src_number =
            self.data
                .try_get_column_index(&args[0])
                .ok_or(CedError::InvalidColumn(format!(
                    "Column : \"{}\" is not valid",
                    args[0]
                )))?;
        let target_number =
            self.data
                .try_get_column_index(&args[1])
                .ok_or(CedError::InvalidColumn(format!(
                    "Column : \"{}\" is not valid",
                    args[1]
                )))?;
        self.move_column(src_number, target_number)?;
        utils::write_to_stdout("Column moved\n")?;
        Ok(())
    }

    fn rename_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CliError(format!(
                "Insufficient arguments for rename-column"
            )));
        }

        let column = &args[0];
        let new_name = &args[1];

        self.rename_column(&column, &new_name)?;
        utils::write_to_stdout("Column renamed\n")?;
        Ok(())
    }

    /// Edit single row
    fn edit_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let len = args.len();
        let row_number: usize;
        let values;
        match len {
            0 => { // No row
                return Err(CedError::CliError(format!(
                            "Insufficient arguments for edit-row"
                )));
            }
            1 => { // Only row
                row_number = args[0].parse::<usize>().map_err(|_| {
                    CedError::CliError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                values = self.edit_row_loop(row_number)?;
            }
            _ => { // From 2.. row + data
                row_number = args[0].parse::<usize>().map_err(|_| {
                    CedError::CliError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                values = args[1]
                    .split(",")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            }
        }

        self.edit_row(row_number, &values)?;
        utils::write_to_stdout("Row content changed\n")?;
        Ok(())
    }

    /// Multiple rows
    fn edit_rows_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let len = args.len();
        let mut start_index = 0;
        let mut end_index = self.get_row_count().max(1) - 1;
        match len {
            // No row
            0 => { }
            1 => { // Only starting row
                start_index = args[0].parse::<usize>().map_err(|_| {
                    CedError::CliError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
            }
            _ => { // From 2.. Starting row + ending row
                start_index = args[0].parse::<usize>().map_err(|_| {
                    CedError::CliError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                end_index = args[1].parse::<usize>().map_err(|_| {
                    CedError::CliError(format!("\"{}\" is not a valid row number", args[1]))
                })?;
            }
        }

        // Inclusive range
        for index in start_index..=end_index {
            let values = self.edit_row_loop(index)?;
            self.edit_row(index, &values)?;
        }

        utils::write_to_stdout("Rows' contents changed\n")?;
        Ok(())
    }

    fn edit_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CliError(format!(
                "Insufficient arguments for edit-column"
            )));
        }

        let column = &args[0];
        let new_value = &args[1];

        self.edit_column(column, new_value)?;
        utils::write_to_stdout("Column content changed\n")?;
        Ok(())
    }

    fn edit_cell_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 1 {
            return Err(CedError::CliError(format!("Edit needs coordinate")));
        }

        let coord = &args[0].split(',').collect::<Vec<&str>>();
        let value = if args.len() >= 2 { args[1..].join(" ") } else { String::new() };
        if coord.len() != 2 {
            return Err(CedError::CliError(format!(
                "Cell cooridnate should be in a form of \"row,column\""
            )));
        }

        let row = coord[0].parse::<usize>().map_err(|_| {
            CedError::CliError(format!("\"{}\" is not a valid row number", coord[0]))
        })?;
        let column = self
            .data
            .try_get_column_index(coord[1])
            .ok_or(CedError::InvalidColumn(format!(
                "Column : \"{}\" is not valid",
                coord[1]
            )))?;

        self.edit_cell(row, column, &value)?;
        utils::write_to_stdout("Cell content changed\n")?;
        Ok(())
    }

    fn add_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let len = args.len();
        let row_number: usize;
        let values;
        match len {
            0 => { // No row
                row_number = self.get_row_count();
                if row_number > self.get_row_count() {
                    return Err(CedError::InvalidColumn(format!(
                                "Cannot add row to out of range position : {}", row_number
                    )));
                }
                values = self.add_row_loop()?;
            }
            1 => { // Only row
                row_number = args[0].parse::<usize>().map_err(|_| {
                    CedError::CliError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                if row_number > self.get_row_count() {
                    return Err(CedError::InvalidColumn(format!(
                                "Cannot add row to out of range position : {}", row_number
                    )));
                }
                values = self.add_row_loop()?;
            }
            _ => { // From 2.. row + data
                row_number = args[0].parse::<usize>().map_err(|_| {
                    CedError::CliError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                values = args[1]
                    .split(",")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            }
        }
        self.add_row_from_strings(row_number, &values)?;
        utils::write_to_stdout("New row added\n")?;
        Ok(())
    }

    fn add_row_loop(&self) -> CedResult<Vec<String>> {
        let mut values = vec![];
        for col in &self.data.columns {
            let default = col.get_default_value();
            utils::write_to_stdout(&format!("{}~{{{}}} = ", col.name, default))?;
            let value = utils::read_stdin(true)?;
            let value = if value.len() != 0 {
                value
            } else {
                default.to_string()
            };

            values.push(value);
        }
        Ok(values)
    }

    fn edit_row_loop(&self, row_number: usize) -> CedResult<Vec<String>> {
        let mut values = vec![];
        for (idx,col) in self.data.columns.iter().enumerate() {
            let default = self.get_cell(row_number,idx)?.ok_or(CedError::OutOfRangeError)?;
            utils::write_to_stdout(&format!("{}~{{{}}} = ", col.name, default))?;
            let value = utils::read_stdin(true)?;
            let value = if value.len() != 0 {
                value
            } else {
                default.to_string()
            };

            values.push(value);
        }
        Ok(values)
    }

    fn add_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let column_name;
        let mut column_number = self.data.get_column_count();
        let mut column_type = ValueType::Text;
        let mut placeholder = None;

        if args.len() == 0 {
            return Err(CedError::CliError(format!(
                "Cannot add column without name"
            )));
        }

        column_name = args[0].as_str();

        if args.len() >= 2 {
            column_number = args[1].parse::<usize>().map_err(|_| {
                CedError::CliError(format!("\"{}\" is not a valid column number", args[1]))
            })?;
        }
        if args.len() >= 3 {
            column_type = ValueType::from_str(&args[2]);
        }

        if args.len() >= 4 {
            placeholder.replace(Value::from_str(&args[3], column_type)?);
        }

        self.add_column(column_number, column_name, column_type, None, placeholder)?;
        utils::write_to_stdout("New column added\n")?;
        Ok(())
    }

    fn remove_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let row_count = if args.len() == 0 {
            self.get_row_count()
        } else {
            args[0]
                .parse::<usize>()
                .map_err(|_| CedError::CliError(format!("\"{}\" is not a valid index", args[0])))?
        }
        .sub(1);

        if let None = self.remove_row(row_count) {
            utils::write_to_stderr("No such row to remove\n")?;
        }
        utils::write_to_stdout("A row removed\n")?;
        Ok(())
    }

    fn remove_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let column_count = if args.len() == 0 {
            self.data.get_column_count()
        } else {
            self.data
                .try_get_column_index(&args[0])
                .ok_or(CedError::InvalidColumn(format!("")))?
        };

        self.remove_column(column_count)?;
        utils::write_to_stdout("A column removed\n")?;
        Ok(())
    }

    fn print_help_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() == 0 {
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
    fn import_file_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        match args.len() {
            0 => {
                return Err(CedError::CliError(format!(
                    "You have to specify a file name to import from"
                )))
            }
            1 => self.import_from_file(Path::new(&args[0]), true)?,
            _ => self.import_from_file(
                Path::new(&args[0]),
                args[1].parse().map_err(|_| {
                    CedError::CliError(format!(
                        "Given value \"{}\" shoul be a valid boolean value. ( has_header )",
                        args[1]
                    ))
                })?,
            )?,
        }
        utils::write_to_stdout("File imported\n")?;
        Ok(())
    }

    fn import_schema_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CliError(format!(
                "Insufficient variable for schema"
            )));
        }
        let force = &args[1];
        self.set_schema(
            &args[0],
            !force
                .parse::<bool>()
                .map_err(|_| CedError::CliError(format!("{force} is not a valid value")))?,
        )?;
        utils::write_to_stdout("Schema applied\n")?;
        Ok(())
    }

    fn export_schema_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 1 {
            return Err(CedError::CliError(format!(
                "Schema export needs a file path"
            )));
        }
        let file = &args[0];
        let mut file = std::fs::File::create(file)
            .map_err(|err| CedError::io_error(err, "Failed to open file for schema write"))?;

        file.write_all(self.export_schema().as_bytes())
            .map_err(|err| CedError::io_error(err, "Failed to write schema to a file"))?;

        utils::write_to_stdout("Schema exported\n")?;
        Ok(())
    }

    fn init_schema_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let file_name = if args.len() < 1 {
            "ced_schema.csv"
        } else {
            &args[0]
        };
        let mut file = std::fs::File::create(file_name)
            .map_err(|err| CedError::io_error(err, "Failed to open file for schema init"))?;

        file.write_all(SCHEMA_HEADER.as_bytes())
            .map_err(|err| CedError::io_error(err, "Failed to write schema to a file"))?;

        utils::write_to_stdout("Schema initiated\n")?;
        Ok(())
    }

    fn write_to_file_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 1 {
            return Err(CedError::CliError(format!("Export requires file path")));
        }
        self.write_to_file(&args[0])?;
        utils::write_to_stdout("File exported\n")?;
        Ok(())
    }

    fn overwrite_to_file_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let cache: bool;
        if args.len() >= 1 {
            cache = args[0].parse::<bool>().map_err(|_| {
                CedError::CliError(format!("\"{}\" is not a valid boolean value", args[0]))
            })?;
        } else {
            cache = true;
        }
        self.overwrite_to_file(cache)?;
        utils::write_to_stdout("File overwritten\n")?;
        Ok(())
    }

    // Combine ths with viewer variant
    fn print(&self, args: &Vec<String>) -> CedResult<()> {
        let csv = self.data.to_string();
        let mut viewer = vec![];
        // Use given command 
        // or use environment variable
        // External command has higher priority
        if args.len() >= 1 {
            viewer = args[0..].to_vec();
        } else if let Ok(var) = std::env::var("CED_VIEWER") {
            viewer = var.split_whitespace().map(|s| s.to_string()).collect(); 
        }

        if viewer.len() == 0 {
            self.print_with_numbers(&csv)?;
        } else {
            self.print_with_viewer(csv, &viewer)?;
        }

        Ok(())
    }

    fn print_cell(&self, args: &Vec<String>) -> CedResult<()> {
        if args.len() == 0 {
            return Err(CedError::CliError(format!(
                "Cannot print cell without a cooridnate"
            )));
        }
        let mut print_mode = "simple";
        let coord = args[0].split(',').collect::<Vec<&str>>();
        let (x,y) = (
            coord[0].parse::<usize>().map_err(|_| {
                    CedError::CliError("You need to feed usize number for coordinate".to_string())
            })?,
            self.data.try_get_column_index(coord[1]).ok_or(CedError::CliError("You need to appropriate column for coordinate".to_string())
            )?
        );

        if args.len() >= 2 {
            print_mode = &args[1];
        }

        match self.data.get_cell(x,y)? {
            Some(cell) => {
                match print_mode {
                    "v" | "verbose" => utils::write_to_stdout(&format!("{:?}\n", cell))?,
                    "d" | "debug" => {
                        let col = self.get_column(y).ok_or(CedError::OutOfRangeError)?;
                        utils::write_to_stdout(&format!("{:#?}\n", col))?;
                        utils::write_to_stdout(&format!("Cell data : {:#?}\n", cell))?
                    },
                    _ => utils::write_to_stdout(&format!("{}\n", cell))?,
                }
            },
            None => utils::write_to_stdout("No such cell\n")?,
        }

        Ok(())
    }

    fn print_column(&self, args: &Vec<String>) -> CedResult<()> {
        let mut print_mode = "simple";
        if args.len() == 0 {
            return Err(CedError::CliError(format!(
                "Cannot print column without name"
            )));
        }

        if args.len() >= 2 {
            print_mode = &args[1];
        }

        if let Some(col) = self.data.try_get_column_index(&args[0]) {
            if col < self.data.get_column_count() {
                let col = self.get_column(col).ok_or(CedError::OutOfRangeError)?;
                match print_mode {
                    "debug"|"d" => utils::write_to_stdout(&format!("{:#?}\n", col))?,
                    "verbose" |"v" => utils::write_to_stdout(&format!("Column = \nName: {}\nType: {}\n---\nLimiter =\n{}", col.get_name(), col.get_column_type(), col.limiter))?,
                    _ => utils::write_to_stdout(&format!("Name: {}\nType: {}\n", col.get_name(), col.get_column_type()))?,
                } 
                return Ok(());
            }
        }
        utils::write_to_stdout("No such column\n")?;
        Ok(())
    }

    fn print_with_viewer(&self, csv: String, viewer: &[String]) -> CedResult<()> {
        let mut process = std::process::Command::new(&viewer[0])
            .args(&viewer[1..])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| {
                CedError::CliError(format!("Failed to execute print command : \"{}\"", &viewer[0]))
            })?;
        let mut stdin = process
            .stdin
            .take()
            .ok_or(CedError::CliError("Failed to read from stdin".to_string()))?;
        std::thread::spawn(move || {
            stdin
                .write_all(csv.as_bytes())
                .expect("Failed to write to stdin");
        });
        let output = process
            .wait_with_output()
            .map_err(|_| CedError::CliError("Failed to write to stdout".to_string()))?;
        let out_content = String::from_utf8_lossy(&output.stdout);
        let err_content = String::from_utf8_lossy(&output.stderr);

        if out_content.len() != 0 {
            utils::write_to_stdout(&out_content)?;
        }
        if err_content.len() != 0 {
            utils::write_to_stderr(&err_content)?;
        }
        Ok(())
    }

    fn print_with_numbers(&self, csv: &str) -> CedResult<()> {
        // Empty csv value, return early
        if csv.len() == 0 {
            utils::write_to_stderr(": CSV is empty :\n")?;
            return Ok(());
        }
        let mut iterator = csv.lines().enumerate();

        // 0 length csv was exited at this moment, thus safe to unwrap
        let header = iterator.next().unwrap().1;
        let header_with_number = format!(
            "-> {}\n",
            header
                .split(',')
                .enumerate()
                .map(|(i, h)| format!("[{}]-{}", i, h))
                .collect::<Vec<String>>()
                .join(",")
        );
        utils::write_to_stdout(&header_with_number)?;

        for (index, line) in iterator {
            if index == 0 {
                utils::write_to_stdout(&format!("- | {}\n", line))?;
            } else {
                utils::write_to_stdout(&format!("{} | {}\n", index - 1, line))?;
            }
        }
        Ok(())
    }

    pub fn limit_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() == 0 {
            self.add_limiter_prompt()?;
        } else {
            let source = args[0..].to_vec().join(" ").split(",").map(|s| s.to_string()).collect::<Vec<_>>();
            if source.len() < 6 {
                return Err(CedError::CliError(format!(
                            "Limit argument needs column_name,type,default,variant,pattern,force(bool)"
                )));
            }

            let col = &source[0];
            let force_str = &source[5];
            let force = if force_str.is_empty() {
                true
            } else {
                force_str.parse::<bool>().map_err(|_| {
                    CedError::CliError("You need to feed boolean value for update".to_string())
                })?
            };
            let limiter = ValueLimiter::from_line(&source[1..5].to_vec())?;
            self.set_limiter(&col, limiter, !force)?;
        }
        Ok(())
    }

    fn add_limiter_prompt(&mut self) -> CedResult<()> {
        utils::write_to_stdout("Column = ")?;
        let column = utils::read_stdin(true)?;

        let mut limiter_attributes = vec![];
        utils::write_to_stdout("Type (Text|Number) = ")?;
        limiter_attributes.push(utils::read_stdin(true)?);

        utils::write_to_stdout("Default = ")?;
        limiter_attributes.push(utils::read_stdin(true)?);

        utils::write_to_stdout("Variants(a b c) = ")?;
        limiter_attributes.push(utils::read_stdin(true)?);

        utils::write_to_stdout("Pattern = ")?;
        limiter_attributes.push(utils::read_stdin(true)?);

        utils::write_to_stdout("Force update(default=true) = ")?;
        let force = utils::read_stdin(true)?;

        let force = if force.is_empty() {
            true
        } else {
            force.parse::<bool>().map_err(|_| {
                CedError::CliError("You need to feed boolean value for the force value".to_string())
            })?
        };

        let limiter = ValueLimiter::from_line(&limiter_attributes)?;
        self.set_limiter(&column, limiter, !force)?;
        Ok(())
    }
}
