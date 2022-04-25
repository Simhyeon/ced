use crate::{Command, Processor, utils, CedResult , cli::help};
use crate::cli::parse::{FlagType, Parser};
use crate::command::{CommandHistory, CommandType};

pub fn start_main_loop() -> CedResult<()> {
    let args: Vec<String> = std::env::args().collect();
    let flags = Parser::new().parse_from_vec(&args[1..].to_vec());

    // Start command loop
    let mut command_loop = CommandLoop::new();
    let mut command_exit = false;
    let mut write_confirm = false;

    for item in flags.iter() {
        match item.ftype {
            FlagType::Version => help::print_version(),
            FlagType::Help => help::print_help_text(),
            FlagType::Confirm => write_confirm = true,
            FlagType::Argument => {
                feed_import(&item.option, &mut command_loop)?;
            }
            FlagType::Schema => {
                feed_schema(&item.option, &mut command_loop)?;
            }
            FlagType::Command => {
                feed_command(&item.option, &mut command_loop, write_confirm)?;
                command_exit = true;
            }
            FlagType::None => (),
        }

        if item.early_exit || command_exit { return Ok(()); }
    }

    command_loop
        .start_loop()
        .err()
        .map(|err| println!("{}", err));
    Ok(())
}

fn feed_import(file: &str, command_loop: &mut CommandLoop) -> CedResult<()> {
    if let Err(err) = command_loop.feed_command(&Command::from_str(&format!("import {}", file))?,true) {
        eprintln!("{}",err);
        return Ok(());
    }
    Ok(())
}

fn feed_schema(file: &str, command_loop: &mut CommandLoop) -> CedResult<()> {
    if let Err(err) = command_loop.feed_command(&Command::from_str(&format!("schema {} true", file))?,true) {
        eprintln!("{}",err);
        return Ok(());
    }
    Ok(())
}

fn feed_command(command: &str, command_loop: &mut CommandLoop, write_confirm: bool) -> CedResult<()> {
    let command_split: Vec<&str> = command.split_terminator(";").collect();
    for command in command_split {
        let command = Command::from_str(command)?;
        // Write should confirm
        if command.command_type == CommandType::Write && write_confirm {
            command_loop.feed_command(&Command::from_str("print")?, true)?;
            utils::write_to_stdout("Overwrite ? (y/N) : ")?;
            if utils::read_stdin(true)?.to_lowercase().as_str() != "y" {
                return Ok(());
            }
        }

        if let Err(err) = command_loop.feed_command(&command,true) {
            eprintln!("{}",err);
            return Ok(());
        }
    }
    
    Ok(())
}


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
        utils::write_to_stdout(&format!("{:?}\n", command))?;

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
            | CommandType::Exit
            | CommandType::Import
            | CommandType::Export
            | CommandType::Create
            | CommandType::Write
            | CommandType::None
            | CommandType::Schema
            | CommandType::SchemaInit
            | CommandType::SchemaExport
            | CommandType::PrintCell
            | CommandType::PrintRow
            | CommandType::PrintColumn
            | CommandType::Print => (),

            // Meta related
            CommandType::Help
            | CommandType::Version => (),

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

