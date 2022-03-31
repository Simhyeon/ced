use crate::processor::Processor;
use crate::error::{CedResult, CedError};
use crate::value::ValueType;
use crate::virtual_data::VirtualData;
use super::utils;
use std::{path::Path, ops::Sub};

#[derive(PartialEq, Debug)]
pub enum CommandType {
    Version,
    Help,
    Undo,
    Redo,
    Create,
    Write,
    Import,
    AddRow,
    AddColumn,
    DeleteRow, 
    DeleteColumn,
    EditCell,
    EditColumn,
    RenameColumn,
    EditRow,
    MoveRow,
    MoveColumn,
    Exit,
    Print,
    None,
}

impl CommandType {
    pub fn from_str(src: &str) -> Self {
        let command_type = match src.to_lowercase().trim() {
            "version"           | "v"  => Self::Version,
            "help"              | "h"  => Self::Help,
            "import"            | "i"  => Self::Import,
            "create"            | "c"  => Self::Create,
            "write"             | "w"  => Self::Write,
            "print"             | "p"  => Self::Print,
            "add-row"           | "ar" => Self::AddRow,
            "exit" | "quit"     | "q"  => Self::Exit,
            "add-column"        | "ac" => Self::AddColumn,
            "delete-row"        | "dr" => Self::DeleteRow,
            "delete-column"     | "dc" => Self::DeleteColumn,
            "edit" |"edit-cell" | "e"  => Self::EditCell,
            "edit-row"          | "er" => Self::EditRow,
            "edit-column"       | "ec" => Self::EditColumn,
            "rename-column"     | "rc" => Self::RenameColumn,
            "move-row" | "move" | "m"  => Self::MoveRow,
            "move-column"       | "mc" => Self::MoveColumn,
            "undo"              | "u"  => Self::Undo,
            "redo"              | "r"  => Self::Redo,
            _ => Self::None,
        };
        command_type
    }
}

#[derive(Debug)]
pub struct Command {
    pub command_type : CommandType,
    pub arguments    : Vec<String>,
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
        let src : Vec<&str> = src.split_whitespace().collect();
        let command = src[0];
        let command_type = CommandType::from_str(command);
        Ok(Self {
            command_type,
            arguments : src[1..].iter().map(|s| s.to_string()).collect(),
        })
    }
}

const HISTORY_CAPACITY: usize = 16;

pub struct CommandHistory {
    // TODO
    // Preserved for command pattern 
    // history: Vec<Command>,
    memento_history: Vec<VirtualData>,
    redo_history: Vec<VirtualData>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            memento_history : vec![],
            redo_history    : vec![],
        }
    }

    pub(crate) fn take_snapshot(&mut self, data : &VirtualData) {
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

pub struct CommandLoop {
    history: CommandHistory,
    processor: Processor,
}

impl CommandLoop {
    pub fn new() -> Self {
        Self { 
            history : CommandHistory::new(),
            processor : Processor::new(),
        }
    }

    pub fn start_loop(&mut self) -> CedResult<()> {
        let mut command = Command::default();
        utils::write_to_stdout("Ced, a csv editor\n")?;
        while CommandType::Exit != command.command_type {
            utils::write_to_stdout(">> ")?;
            command = Command::from_str(&utils::read_stdin(true)?)?;

            // DEBUG NOTE TODO
            #[cfg(debug_assertions)]
            println!("{:?}", command);

            match command.command_type {
                CommandType::Undo | CommandType::Redo => {
                    if command.command_type == CommandType::Undo {
                        self.undo()?;
                    } else {
                        self.redo()?;
                    }
                    continue;
                }
                // Un-redoable commands
                CommandType::Help | CommandType::Exit | 
                    CommandType::Import | CommandType::Print | 
                    CommandType::Create | CommandType::Write | 
                    CommandType::None | CommandType::Version => (),
                    _ => self.history.take_snapshot(&self.processor.data),
            }

            if let Err(err) = self.processor.execute_command(&command) {
                utils::write_to_stderr(&(err.to_string() + "\n"))?;
            }
        }
        Ok(())
    }

    fn undo(&mut self) -> CedResult<()> {
        if let Some(prev) =  self.history.pop() {
            self.processor.data = prev.clone();
        }
        Ok(())
    }

    fn redo(&mut self) -> CedResult<()> {
        if let Some(prev) =  self.history.pull_redo() {
            self.processor.data = prev;
        }
        Ok(())
    }
}

impl Processor {
    pub fn execute_command(&mut self, command: &Command) -> CedResult<()> {
        match command.command_type {
            CommandType::Version => utils::write_to_stdout("ced, 0.1.0\n")?,
            CommandType::Help => utils::write_to_stdout(include_str!("../../src/help.txt"))?,
            CommandType::Import => self.import_file_from_args(&command.arguments)?,
            CommandType::Write => self.overwrite_to_file_from_args(&command.arguments)?,
            CommandType::Create => {
                self.add_column_array(&command.arguments);
                utils::write_to_stdout("New columns added\n")?;
            },
            CommandType::Print => self.print()?,
            CommandType::AddRow => self.add_row_from_args(&command.arguments)?,
            CommandType::DeleteRow => self.remove_row_from_args(&command.arguments)?,
            CommandType::DeleteColumn => self.remove_column_from_args(&command.arguments)?,
            CommandType::AddColumn => self.add_column_from_args(&command.arguments)?,
            CommandType::EditCell => self.edit_cell_from_args(&command.arguments)?,
            CommandType::EditRow => self.edit_row_from_args(&command.arguments)?,
            CommandType::EditColumn => self.edit_column_from_args(&command.arguments)?,
            CommandType::RenameColumn => self.rename_column_from_args(&command.arguments)?,
            CommandType::MoveRow => self.move_row_from_args(&command.arguments)?,
            CommandType::MoveColumn => self.move_column_from_args(&command.arguments)?,
            _ => (),
        }
        Ok(())
    }

    fn move_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CliError(format!("Insufficient arguments for move-row")));
        }
        let src_number = args[0].parse::<usize>()
            .map_err(|_|CedError::CliError(format!("\"{}\" is not a valid row number", args[0])))?;
        let target_number = args[1].parse::<usize>()
            .map_err(|_|CedError::CliError(format!("\"{}\" is not a valid row number", args[0])))?;
        self.move_row(src_number,target_number)?;
        utils::write_to_stdout("Row moved\n")?;
        Ok(())
    }

    fn move_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CliError(format!("Insufficient arguments for move-column")));
        }
        let src_number = self.data.get_column_index(&args[0]).ok_or(CedError::InvalidColumn(format!("Column : \"{}\" is not valid", args[0])))?;
        let target_number = self.data.get_column_index(&args[1]).ok_or(CedError::InvalidColumn(format!("Column : \"{}\" is not valid", args[1])))?;
        self.move_column(src_number,target_number)?;
        utils::write_to_stdout("Column moved\n")?;
        Ok(())
    }

    fn rename_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CliError(format!("Insufficient arguments for rename-column")));
        }

        let column = &args[0];
        let new_name = &args[1];

        self.rename_column(&column, &new_name)?;
        utils::write_to_stdout("Column renamed\n")?;
        Ok(())
    }

    fn edit_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CliError(format!("Insufficient arguments for edit-row")));
        }

        let row_number = args[0].parse::<usize>()
            .map_err(|_|CedError::CliError(format!("\"{}\" is not a valid row number", args[0])))?;
        let row_data = &args[1];

        if row_number >= self.data.get_row_count() {
            return Err(CedError::CliError(format!("Row number out of bounds")));
        } 

        self.edit_row(row_number, row_data)?;
        utils::write_to_stdout("Row content changed\n")?;
        Ok(())
    }

    fn edit_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CliError(format!("Insufficient arguments for edit-column")));
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
        let value = if args.len() >= 2{&args[1]} else { "" };
        if coord.len() != 2 {
            return Err(CedError::CliError(format!("Cell cooridnate should be in a form of \"row,column\"")));
        }

        let row = coord[0].parse::<usize>().map_err(|_|CedError::CliError(format!("\"{}\" is not a valid row number", coord[0])))?;
        let column = self.data.get_column_index(coord[1]).ok_or(CedError::InvalidColumn(format!("Column : \"{}\" is not valid", coord[1])))?;

        self.edit_cell(row, column, value)?;
        utils::write_to_stdout("Cell content changed\n")?;
        Ok(())
    }

    fn add_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let len = args.len();
        let row_number : usize;
        let values;
        match len {
            0 => {
                row_number = self.data.get_row_count();
                values =  self.add_row_loop()?;
            }
            1 => {
                row_number = args[0].parse::<usize>().map_err(|_|CedError::CliError(format!("\"{}\" is not a valid row number", args[1])))?; 
                values =  self.add_row_loop()?;
            }
            _ => { // From 2..
                row_number = args[0].parse::<usize>().map_err(|_|CedError::CliError(format!("\"{}\" is not a valid row number", args[1])))?; 
                values = args[1].split(",").map(|s|s.to_string()).collect::<Vec<String>>()
            }
        }
        self.add_row_from_strings(row_number,&values)?;
        utils::write_to_stdout("New row added\n")?;
        Ok(())
    }

    fn add_row_loop(&self) -> CedResult<Vec<String>> {
        let mut values = vec![];
        for col in &self.data.columns {
            utils::write_to_stdout(&format!("{} = ", col.name))?;
            values.push(utils::read_stdin(true)?);
        }
        Ok(values)
    }

    fn add_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let column_name;
        let mut column_number = self.data.get_column_count();
        let mut column_type = ValueType::Text;

        if args.len() == 0 {
            return Err(CedError::CliError(format!("Cannot add column without name")));
        }

        column_name = args[0].as_str();

        if args.len() >= 2 {
            column_number = args[1].parse::<usize>().map_err(|_|CedError::CliError(format!("\"{}\" is not a valid column number", args[1])))?;
        }
        if args.len() >= 3 {
            column_type = ValueType::from_str(&args[2]);
        }

        self.add_column(column_number, column_name, column_type, None);
        utils::write_to_stdout("New column added\n")?;
        Ok(())
    }

    fn remove_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let row_count = if args.len() == 0 {
            self.data.get_row_count()
        } else {
            args[0].parse::<usize>().map_err(|_| CedError::CliError(format!("\"{}\" is not a valid index", args[0])))?
        }.sub(1);

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
            args[0].parse::<usize>().map_err(|_| CedError::CliError(format!("\"{}\" is not a valid index", args[0])))?
        }.sub(1);

        self.remove_column(column_count)?;
        utils::write_to_stdout("A column removed\n")?;
        Ok(())
    }

    /// Read from args
    ///
    /// file is asssumed to have header
    /// You can give has_header value as second parameter
    fn import_file_from_args(&mut self, args : &Vec<String>) -> CedResult<()> {
        match args.len() {
            0 => return Err(CedError::CliError(format!("You have to specify a file name to import from"))),
            1 => self.import_from_file(Path::new(&args[0]), true)?,
            _ => self.import_from_file(Path::new(&args[0]), args[1].parse().map_err(|_| CedError::CliError(format!("Given value \"{}\" shoul be a valid boolean value. ( has_header )", args[1])))?)?,
        
        }
        utils::write_to_stdout("File imported\n")?;
        Ok(())
    }

    fn overwrite_to_file_from_args(&mut self, args : &Vec<String>) -> CedResult<()> {
        let cache: bool;
        if args.len() >= 1 {
            cache = args[0].parse::<bool>()
                .map_err(|_|CedError::CliError(format!("\"{}\" is not a valid boolean value", args[0])))?;
        } else {
            cache = true;
        }
        self.overwrite_to_file(cache)?;
        utils::write_to_stdout("File overwritten\n")?;
        Ok(())
    }

    // Combine ths with viewer variant
    fn print(&self) -> CedResult<()> {
        let csv = self.data.to_string();
        // TODO
        // self.print_with_printer(&csv_src);
        self.print_with_numbers(&csv)?;

        Ok(())
    }

    fn print_with_numbers(&self, csv: &str) -> CedResult<()> {
        // Empty csv value, return early
        if csv.len() == 0 {
            println!(": CSV is empty :");
            return Ok(());
        }
        // TODO
        // Print columns numbers
        // Print row numbers
        let mut iterator = csv.lines().enumerate();

        let header = iterator.next().unwrap().1;
        let header_with_number = format!("-> {}\n",header.split(',').enumerate().map(|(i,h)| format!("[{}]-{}",i,h)).collect::<Vec<String>>().join(","));
        utils::write_to_stdout(&header_with_number)?;

        for (index, line) in iterator {
            if index == 0 {
                utils::write_to_stdout(&format!("- | {}\n",line))?;
            } else {
                utils::write_to_stdout(&format!("{} | {}\n",index-1,line))?;
            }
        }
        Ok(())
    }
}
