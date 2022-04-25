use crate::utils::{self,subprocess};
use crate::error::{CedError, CedResult};
use crate::processor::Processor;
use crate::value::ValueType;
use crate::virtual_data::{VirtualData, SCHEMA_HEADER, Row, Column};
use crate::{ValueLimiter, Value};
#[cfg(feature = "cli")]
use crate::cli::help;
use std::io::Write;
use std::{ops::Sub, path::Path};

#[derive(PartialEq, Debug)]
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
    PrintRow,
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
            #[cfg(feature = "cli")]
            "version" | "v" => Self::Version,
            #[cfg(feature = "cli")]
            "help" | "h" => Self::Help,
            "import" | "i" => Self::Import,
            "export" | "x" => Self::Export,
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

// Default history capacity double word
const HISTORY_CAPACITY: usize = 16;
pub struct CommandHistory {
    // TODO
    // Preserved for command pattern
    // history: Vec<Command>,
    memento_history: Vec<VirtualData>,
    redo_history: Vec<VirtualData>,
    history_capacity: usize,
}

impl CommandHistory {
    pub fn new() -> Self {
        let capacity = if let Ok(cap) = std::env::var("CED_HISTORY_CAPACITY") {
            if let Ok(num) = cap.parse::<usize>() { 
                num
            } else { HISTORY_CAPACITY }
        } else { HISTORY_CAPACITY };
        Self {
            memento_history: vec![],
            redo_history: vec![],
            history_capacity: capacity,
        }
    }

    pub(crate) fn take_snapshot(&mut self, data: &VirtualData) {
        self.memento_history.push(data.clone());
        // You cannot redo if you have done something other than undo
        self.redo_history.clear();
        if self.memento_history.len() > self.history_capacity {
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
impl Processor {
    /// Execute given command
    pub fn execute_command(&mut self, command: &Command) -> CedResult<()> {
        match command.command_type {
            #[cfg(feature = "cli")]
            CommandType::Version => help::print_version(),
            #[cfg(feature = "cli")]
            CommandType::Help => self.print_help_from_args(&command.arguments)?,
            CommandType::None => utils::write_to_stderr("No such command \n")?,
            CommandType::Import => self.import_file_from_args(&command.arguments)?,
            CommandType::Schema => self.import_schema_from_args(&command.arguments)?,
            CommandType::SchemaInit => self.init_schema_from_args(&command.arguments)?,
            CommandType::SchemaExport => self.export_schema_from_args(&command.arguments)?,
            CommandType::Export => self.write_to_file_from_args(&command.arguments)?,
            CommandType::Write => self.overwrite_to_file_from_args(&command.arguments)?,
            CommandType::Create => {
                self.add_column_array(&command.arguments)?;
                self.log("New columns added\n")?;
            }
            CommandType::Print => self.print(&command.arguments)?,
            CommandType::PrintCell => self.print_cell(&command.arguments)?,
            CommandType::PrintRow => self.print_row(&command.arguments)?,
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
            return Err(CedError::CommandError(format!(
                "Insufficient arguments for move-row"
            )));
        }
        let src_number = args[0].parse::<usize>().map_err(|_| {
            CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
        })?;
        let target_number = args[1].parse::<usize>().map_err(|_| {
            CedError::CommandError(format!("\"{}\" is not a valid row number", args[1]))
        })?;
        self.move_row(src_number, target_number)?;
        self.log(&format!("Row moved from \"{}\" to \"{}\"\n", src_number, target_number))?;
        Ok(())
    }

    fn move_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CommandError(format!(
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
        self.log(&format!("Column moved from \"{}\" to \"{}\"\n", src_number, target_number))?;
        Ok(())
    }

    fn rename_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CommandError(format!(
                "Insufficient arguments for rename-column"
            )));
        }

        let column = &args[0];
        let new_name = &args[1];

        self.rename_column(&column, &new_name)?;
        self.log(&format!("Column renamed from \"{}\" to \"{}\"\n", column, new_name))?;
        Ok(())
    }

    /// Edit single row
    fn edit_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let len = args.len();
        let row_number: usize;
        match len {
            0 => { // No row
                return Err(CedError::CommandError(format!(
                            "Insufficient arguments for edit-row"
                )));
            }
            1 => { // Only row
                row_number = args[0].parse::<usize>().map_err(|_| {
                    CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                utils::write_to_stdout("Type comma(,) to exit input\n")?;
                let values = self.edit_row_loop(Some(row_number))?;
                if values.len() == 0 {
                    return Ok(());
                }
                self.edit_row(row_number, values)?;
            }
            _ => { // From 2.. row + data
                row_number = args[0].parse::<usize>().map_err(|_| {
                    CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                let values = args[1]
                    .split(",")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                self.set_row_from_string(row_number, &values)?;
            }
        }

        self.log(&format!("Row \"{}\"'s' content changed\n", row_number))?;
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
                    CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
            }
            _ => { // From 2.. Starting row + ending row
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
            let values = self.edit_row_loop(Some(index))?;
            if values.len() == 0 {
                return Ok(());
            }
            edit_target_values.push((index,values));
        }

        // Edit rows only if every operation was succesfull
        for (index,values) in edit_target_values {
            self.edit_row(index, values)?;
        }

        self.log(&format!("Rows {}~{} contents changed\n", start_index, end_index))?;
        Ok(())
    }

    fn edit_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CommandError(format!(
                "Insufficient arguments for edit-column"
            )));
        }

        let column = &args[0];
        let new_value = &args[1];

        self.edit_column(column, new_value)?;
        self.log(&format!("Column \"{}\" content changed to \"{}\"\n", column, new_value))?;
        Ok(())
    }

    fn edit_cell_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 1 {
            return Err(CedError::CommandError(format!("Edit needs coordinate")));
        }

        let coord = &args[0].split(',').collect::<Vec<&str>>();
        let value = if args.len() >= 2 { args[1..].join(" ") } else { String::new() };
        if coord.len() != 2 {
            return Err(CedError::CommandError(format!(
                "Cell cooridnate should be in a form of \"row,column\""
            )));
        }

        let row = coord[0].parse::<usize>().map_err(|_| {
            CedError::CommandError(format!("\"{}\" is not a valid row number", coord[0]))
        })?;
        let column = self
            .data
            .try_get_column_index(coord[1])
            .ok_or(CedError::InvalidColumn(format!(
                "Column : \"{}\" is not valid",
                coord[1]
            )))?;

        self.edit_cell(row, column, &value)?;
        self.log(&format!("Cell \"({},{})\" content changed \"{}\"\n", row,coord[1],&&value))?;
        Ok(())
    }

    fn add_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let len = args.len();
        let row_number: usize;
        match len {
            0 => { // No row number
                row_number = self.get_row_count();
                if row_number > self.get_row_count() {
                    return Err(CedError::InvalidColumn(format!(
                                "Cannot add row to out of range position : {}", row_number
                    )));
                }
                utils::write_to_stdout("Type comma(,) to exit input\n")?;
                let values = self.add_row_loop(None)?;
                if values.len() == 0 {
                    return Ok(());
                }
                self.add_row(row_number, Some(&values))?;
            }
            1 => { // Only row number
                row_number = args[0].parse::<usize>().map_err(|_| {
                    CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                if row_number > self.get_row_count() {
                    return Err(CedError::InvalidColumn(format!(
                                "Cannot add row to out of range position : {}", row_number
                    )));
                }
                utils::write_to_stdout("Type comma(,) to exit input\n")?;
                let values = self.add_row_loop(None)?;
                if values.len() == 0 {
                    return Ok(());
                }
                self.add_row(row_number, Some(&values))?;
            }
            _ => { // From 2.. row + data
                row_number = args[0].parse::<usize>().map_err(|_| {
                    CedError::CommandError(format!("\"{}\" is not a valid row number", args[0]))
                })?;
                let values = args[1]
                    .split(",")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                self.add_row_from_strings(row_number, &values)?;
            }
        }
        self.log(&format!("New row added to \"{}\"\n", row_number))?;
        Ok(())
    }

    fn add_row_loop(&mut self, row_number: Option<usize>) -> CedResult<Vec<Value>> {
        let mut values = vec![];
        for (idx,col) in self.data.columns.iter().enumerate() {
            let mut value : Value;
            let mut type_mismatch = false;
            let default = if let Some(row_number) = row_number {
                self.get_cell(row_number,idx)?.ok_or(CedError::OutOfRangeError)?.to_owned()
            } else {
                col.get_default_value()
            };
            utils::write_to_stdout(&format!("{}~{{{}}} = ", col.name, default))?;
            let value_src = utils::read_stdin(true)?;
            value = if value_src.len() != 0 {
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

            // Early return
            if let Value::Text(content) = &value {
                if content.contains(",") {
                    return Ok(vec!());
                }
            }

            while !col.limiter.qualify(&value) || type_mismatch {
                type_mismatch = false;
                utils::write_to_stdout("Given value doesn't qualify column limiter\n")?;
                utils::write_to_stdout(&format!("{}~{{{}}} = ", col.name, default))?;
                let value_src = utils::read_stdin(true)?;
                value = if value_src.len() != 0 {
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

                // Early return
                if let Value::Text(content) = &value {
                    if content.contains(",") {
                        return Ok(vec!());
                    }
                }
            }

            values.push(value);
        }
        Ok(values)
    }

    // None means value should not change 
    fn edit_row_loop(&mut self, row_number: Option<usize>) -> CedResult<Vec<Option<Value>>> {
        let mut values = vec![];
        for (idx,col) in self.data.columns.iter().enumerate() {
            let mut value : Option<Value>;
            let mut type_mismatch = false;
            let default = if let Some(row_number) = row_number {
                self.get_cell(row_number,idx)?.ok_or(CedError::OutOfRangeError)?.to_owned()
            } else {
                col.get_default_value()
            };
            utils::write_to_stdout(&format!("{}~{{{}}} = ", col.name, default))?;
            let value_src = utils::read_stdin(true)?;
            value = if value_src.len() != 0 {
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
            if let Some(value) = &value {
                if let Value::Text(content) = value {
                    if content.contains(",") {
                        return Ok(vec!());
                    }
                }
            }

            while value != None { // when value was not a "Not changed"
                if col.limiter.qualify(&value.as_ref().unwrap()) && !type_mismatch {
                    break;
                }
                type_mismatch = false;
                utils::write_to_stdout("Given value doesn't qualify column limiter\n")?;
                utils::write_to_stdout(&format!("{}~{{{}}} = ", col.name, default))?;
                let value_src = utils::read_stdin(true)?;
                value = if value_src.len() != 0 {
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
                if let Value::Text(content) = value.as_ref().unwrap() {
                    if content.contains(",") {
                        return Ok(vec!());
                    }
                }
            }

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
            return Err(CedError::CommandError(format!(
                "Cannot add column without name"
            )));
        }

        column_name = args[0].as_str();

        if args.len() >= 2 {
            column_number = args[1].parse::<usize>().map_err(|_| {
                CedError::CommandError(format!("\"{}\" is not a valid column number", args[1]))
            })?;
        }
        if args.len() >= 3 {
            column_type = ValueType::from_str(&args[2]);
        }

        if args.len() >= 4 {
            placeholder.replace(Value::from_str(&args[3], column_type)?);
        }

        self.add_column(column_number, column_name, column_type, None, placeholder)?;
        self.log(&format!("New column \"{}\" added to \"{}\"\n", column_name, column_number))?;
        Ok(())
    }

    fn remove_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let row_count = if args.len() == 0 {
            self.get_row_count()
        } else {
            args[0]
                .parse::<usize>()
                .map_err(|_| CedError::CommandError(format!("\"{}\" is not a valid index", args[0])))?
        }
        .sub(1);

        if let None = self.remove_row(row_count) {
            utils::write_to_stdout("No such row to remove\n")?;
        }
        self.log(&format!("A row removed from \"{}\"\n", row_count))?;
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
        self.log(&format!("A column \"{}\" removed\n", &args[0]))?;
        Ok(())
    }

    #[cfg(feature = "cli")]
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
                return Err(CedError::CommandError(format!(
                    "You have to specify a file name to import from"
                )))
            }
            1 => self.import_from_file(Path::new(&args[0]), true)?,
            _ => self.import_from_file(
                Path::new(&args[0]),
                args[1].parse().map_err(|_| {
                    CedError::CommandError(format!(
                        "Given value \"{}\" shoul be a valid boolean value. ( has_header )",
                        args[1]
                    ))
                })?,
            )?,
        }
        self.log(&format!("File \"{}\" imported\n", &args[0]))?;
        Ok(())
    }

    fn import_schema_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CommandError(format!(
                "Insufficient variable for schema"
            )));
        }
        let schema_file = &args[0];
        let force = &args[1];
        self.set_schema(
            schema_file,
            !force
                .parse::<bool>()
                .map_err(|_| CedError::CommandError(format!("{force} is not a valid value")))?,
        )?;
        self.log(&format!("Schema \"{}\" applied\n", &args[0]))?;
        Ok(() )
    }

    fn export_schema_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 1 {
            return Err(CedError::CommandError(format!(
                "Schema export needs a file path"
            )));
        }
        let file = &args[0];
        let mut file = std::fs::File::create(file)
            .map_err(|err| CedError::io_error(err, "Failed to open file for schema write"))?;

        file.write_all(self.export_schema().as_bytes())
            .map_err(|err| CedError::io_error(err, "Failed to write schema to a file"))?;

        self.log(&format!("Schema exported to \"{}\"\n",args[0]))?;
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

        self.log(&format!("Schema initiated to \"{}\"\n", file_name))?;
        Ok(())
    }

    fn write_to_file_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 1 {
            return Err(CedError::CommandError(format!("Export requires file path")));
        }
        self.write_to_file(&args[0])?;
        self.log(&format!("File exported to \"{}\"\n", &args[0]))?;
        Ok(())
    }

    fn overwrite_to_file_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let cache: bool;
        if args.len() >= 1 {
            cache = args[0].parse::<bool>().map_err(|_| {
                CedError::CommandError(format!("\"{}\" is not a valid boolean value", args[0]))
            })?;
        } else {
            cache = true;
        }
        let success = self.overwrite_to_file(cache)?;
        if success {self.log(&format!("File overwritten to \"{}\"\n", self.file.as_ref().unwrap().display()))?;}
        Ok(())
    }

    // Combine ths with viewer variant
    fn print(&mut self, args: &Vec<String>) -> CedResult<()> {
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

    fn print_cell(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() == 0 {
            return Err(CedError::CommandError(format!(
                "Cannot print cell without a cooridnate"
            )));
        }
        let mut print_mode = "simple";
        let coord = args[0].split(',').collect::<Vec<&str>>();
        let (x,y) = (
            coord[0].parse::<usize>().map_err(|_| {
                    CedError::CommandError("You need to feed usize number for coordinate".to_string())
            })?,
            self.data.try_get_column_index(coord[1]).ok_or(CedError::CommandError("You need to appropriate column for coordinate".to_string())
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

    fn print_row(&self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 1 {
            return Err(CedError::CommandError(format!("Print-row needs row number")));
        }
        let row = self.get_row(args[0].parse::<usize>().map_err(|_| {
            CedError::CommandError("You need to valid number as row number".to_string())
        })?);

        if let None = row {
            return Err(CedError::CommandError(format!("Print-row needs row number")));
        }
        let row = row.unwrap().to_string(&self.data.columns)? + "\n";

        let mut viewer = vec![];
        // Use given command 
        // or use environment variable
        // External command has higher priority
        if args.len() >= 2 {
            viewer = args[1..].to_vec();
        } else if let Ok(var) = std::env::var("CED_VIEWER") {
            viewer = var.split_whitespace().map(|s| s.to_string()).collect(); 
        }

        subprocess(&viewer, Some(row))
    }

    fn print_column(&mut self, args: &Vec<String>) -> CedResult<()> {
        let mut print_mode = "simple";
        if args.len() == 0 {
            return Err(CedError::CommandError(format!(
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
        subprocess(&viewer.to_vec(), Some(csv))
    }

    fn print_with_numbers(&mut self, csv: &str) -> CedResult<()> {
        // Empty csv value, return early
        if csv.len() == 0 {
            utils::write_to_stdout(": CSV is empty :\n")?;
            return Ok(());
        }
        let mut iterator = csv.lines().enumerate();

        // 0 length csv is panicking error at this moment, thus safe to unwrap
        let header = iterator.next().unwrap().1;
        let header_with_number = format!(
            " -> {}\n",
            header
                .split(',')
                .enumerate()
                .map(|(i, h)| format!("[{}]-{}", i, h))
                .collect::<Vec<String>>()
                .join(",")
        );
        utils::write_to_stdout(&header_with_number)?;

        for (index, line) in iterator {
            // Print row column number in row
            let numbered_line = line
                .split(',')
                .enumerate()
                .map(|(i, h)| format!("[{}]-{}", i, h))
                .collect::<Vec<String>>()
                .join(",");
            utils::write_to_stdout(&format!("{} | {}\n", index - 1, numbered_line))?;
        }
        Ok(())
    }

    pub fn limit_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() == 0 {
            self.add_limiter_prompt()?;
        } else {
            let source = args[0..].to_vec().join(" ").split(",").map(|s| s.to_string()).collect::<Vec<_>>();
            if source.len() < 6 {
                return Err(CedError::CommandError(format!(
                            "Limit argument needs column_name,type,default,variant,pattern,force(bool)"
                )));
            }

            let col = &source[0];
            let force_str = &source[5];
            let force = if force_str.is_empty() {
                true
            } else {
                force_str.parse::<bool>().map_err(|_| {
                    CedError::CommandError("You need to feed boolean value for update".to_string())
                })?
            };
            let limiter = ValueLimiter::from_line(&source[1..5].to_vec())?;
            self.set_limiter(&col, limiter, !force)?;
            self.log(&format!("Limited column \"{}\"", col))?;
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
                CedError::CommandError("You need to feed boolean value for the force value".to_string())
            })?
        };

        let limiter = ValueLimiter::from_line(&limiter_attributes)?;
        self.set_limiter(&column, limiter, !force)?;
        Ok(())
    }
}
