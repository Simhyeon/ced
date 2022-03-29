use crate::{processor::Processor, error::{CedResult, CedError}};
use super::utils;
use std::{path::Path, ops::Sub};

// NOTE
// Cursor operation is not necessary because cli is not cursor based
//
// TODO
// Append selection variant
#[derive(PartialEq, Debug)]
pub enum CommandType {
    Undo, // Undo last command
    Redo, // Redo undid command
    Create, // Crate table
    Overwrite,
    Read,
    AddRow,
    AddColumn,
    RemoveRow,
    RemoveColumn,
    ClearCell,
    ClearColumn,
    ClearRow,
    EditCell,
    EditColumn,
    EditRow,
    Exit,
    Print,
    None,
}

impl CommandType {
    pub fn from_str(src: &str) -> Self {
        let command_type = match src.to_lowercase().trim() {
            "read" | "r" => Self::Read,
            "create" => Self::Create,
            "overwrite" | "ow" => Self::Overwrite,
            "print" | "p" => Self::Print,
            "add-row" | "ar"   => Self::AddRow,
            "remove-row" | "rr"   => Self::RemoveRow,
            "add-column" | "ac" => Self::AddColumn,
            "remove-column" | "rc"   => Self::RemoveColumn,
            "edit" | "e" => Self::EditCell,
            "exit" | "x" => Self::Exit,
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
            CommandType::Read => self.read_file_from_args(&command.arguments)?,
            CommandType::Create => self.create_columns_fast(&command.arguments),
            CommandType::Print => self.print()?,
            CommandType::AddRow => self.add_row_from_args(&command.arguments)?,
            CommandType::RemoveRow => self.remove_row_from_args(&command.arguments)?,
            CommandType::Overwrite => self.overwrite()?,
            CommandType::AddColumn => (),
            _ => (),
        }
        Ok(())
    }

    fn add_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        if args.len() == 0 {
            return Err(CedError::CliError(format!("Cannot add empty value to row")));
        }
        self.add_row_from_vector(&args[0].split(",").collect::<Vec<&str>>())?;
        Ok(())
    }

    fn remove_row_from_args(&mut self, args: &Vec<String>) -> CedResult<()> {
        let row_count = if args.len() == 0 {
            self.data.get_row_count()
        } else {
            args[0].parse::<usize>().map_err(|_| CedError::CliError(format!("Argument \"{}\" is not a valid index", args[0])))?
        }.sub(1);

        if let None = self.remove_row(row_count) {
            utils::write_to_stderr("No such row to remove\n")?;
        }
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
        self.print_with_numbers(&csv);

        Ok(())
    }

    fn print_with_numbers(&self, csv: &str) {
        // TODO
        // Print columns numbers
        // Print row numbers
        print!("{}", csv);
    }
}
