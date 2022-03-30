use crate::{processor::Processor, error::{CedResult, CedError}, value::{ValueType, Value}};
use super::utils;
use std::{path::Path, ops::Sub};

// NOTE
// Cursor operation is not necessary because cli is not cursor based
//
// TODO
// Append selection variant
#[derive(PartialEq, Debug)]
pub enum CommandType {
    Version,
    Help,
    Undo, // TODO
    Redo, // TODO
    Create,
    Overwrite,
    Read,
    AddRow,
    AddColumn,
    DeleteRow, 
    DeleteColumn,
    EditCell,
    EditColumn,   // TODO This also has to check if exists : This is "safe" in terms of sanity though
    RenameColumn, // TODO This also has to be transaction based : This one too
    EditRow,      // TODO -> This should be transaction based : But this one... is not sane in transaction
    Exit,
    Print,
    None,
}

impl CommandType {
    pub fn from_str(src: &str) -> Self {
        let command_type = match src.to_lowercase().trim() {
            "version" | "v" => Self::Version,
            "help" | "h" => Self::Help,
            "read" | "r" => Self::Read,
            "create" | "c" => Self::Create,
            "overwrite" | "ow" => Self::Overwrite,
            "print" | "p" => Self::Print,
            "add-row" | "ar"   => Self::AddRow,
            "delete-row" | "dr"   => Self::DeleteRow,
            "add-column" | "ac" => Self::AddColumn,
            "delete-column" | "dc"   => Self::DeleteColumn,
            "edit" | "e" => Self::EditCell,
            "exit" | "x" => Self::Exit,
            "edit-column" | "ec" => Self::EditColumn,
            "edit-row" | "er" => Self::EditRow,
            "rename-column" | "rc" => Self::RenameColumn,
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

pub struct CommandLoop {
    command_history: Vec<Command>,
    processor: Processor,
}

impl CommandLoop {
    pub fn new() -> Self {
        Self { 
            command_history : vec![],
            processor : Processor::new(),
        }
    }

    pub fn start_loop(&mut self) -> CedResult<()> {
        let mut command = Command::default();
        utils::write_to_stdout("Ced, a csv editor\n")?;
        while CommandType::Exit != command.command_type {
            utils::write_to_stdout(">> ")?;
            command = Command::from_str(&utils::read_stdin()?)?;

            // DEBUG NOTE TODO
            #[cfg(debug_assertions)]
            println!("{:?}", command);

            if let Err(err) = self.processor.execute_command(&command) {
                utils::write_to_stderr(&(err.to_string() + "\n"))?;
            }
        }
        Ok(())
    }
}

impl Processor {
    pub fn execute_command(&mut self, command: &Command) -> CedResult<()> {
        match command.command_type {
            CommandType::Version => utils::write_to_stdout("ced, 0.1.0\n")?,
            CommandType::Help => utils::write_to_stdout(include_str!("../../src/help.txt"))?,
            CommandType::Read => self.read_file_from_args(&command.arguments)?,
            CommandType::Overwrite => self.overwrite()?,
            CommandType::Create => self.create_columns_fast(&command.arguments),
            CommandType::Print => self.print()?,
            CommandType::AddRow => self.add_row_from_args(&command.arguments)?,
            CommandType::DeleteRow => self.remove_row_from_args(&command.arguments)?,
            CommandType::DeleteColumn => self.remove_column_from_args(&command.arguments)?,
            CommandType::AddColumn => self.add_column_from_args(&command.arguments)?,
            CommandType::EditCell => self.edit_cell_from_args(&command.arguments)?,
            CommandType::EditRow => self.edit_row_from_args(&command.arguments)?,
            CommandType::EditColumn => self.edit_column_from_args(&command.arguments)?,
            CommandType::RenameColumn => self.rename_column_from_args(&command.arguments)?,
            _ => (),
        }
        Ok(())
    }

    fn rename_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CliError(format!("Insufficient arguments for rename-column")));
        }

        let column = &args[0];
        let new_name = &args[1];

        self.rename_column(&column, &new_name)?;

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

        Ok(())
    }

    fn edit_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() < 2 {
            return Err(CedError::CliError(format!("Insufficient arguments for edit-column")));
        }

        let column = &args[0];
        let new_value = &args[1];

        self.edit_column(column, new_value)?;

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
        let column = coord[1].parse::<usize>().map_err(|_|CedError::CliError(format!("\"{}\" is not a valid column number", coord[1])))?;

        self.edit_cell(row, column, value)?;

        Ok(())
    }

    fn add_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() == 0 {
            return Err(CedError::CliError(format!("Cannot add empty value to row")));
        }
        self.add_row_from_vector(&args[0].split(",").collect::<Vec<&str>>())?;
        Ok(())
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
        Ok(())
    }

    fn remove_column_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let column_count = if args.len() == 0 {
            self.data.get_column_count()
        } else {
            args[0].parse::<usize>().map_err(|_| CedError::CliError(format!("\"{}\" is not a valid index", args[0])))?
        }.sub(1);

        self.remove_column(column_count)?;
        Ok(())
    }

    fn read_file_from_args(&mut self, args : &Vec<String>) -> CedResult<()> {
        match args.len() {
            0 => return Err(CedError::CliError(format!("You have to specify a file name to read from"))),
            1 => self.read_from_file(Path::new(&args[0]), true)?,
            _ => self.read_from_file(Path::new(&args[0]), args[1].parse().map_err(|_| CedError::CliError(format!("Given value \"{}\" shoul be a valid boolean value. ( has_header )", args[1])))?)?,
        
        }

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
