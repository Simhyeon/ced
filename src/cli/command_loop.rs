use crate::cli::parse::{FlagType, Parser};
use crate::command::{CommandHistory, CommandType};
use crate::CedError;
use crate::{cli::help, utils, CedResult, Command, Processor};
use std::str::FromStr;

pub fn start_main_loop() -> CedResult<()> {
    let args: Vec<String> = std::env::args().collect();
    let flags = Parser::new().parse_from_vec(&args[1..].to_vec());
    // Start command loop
    let mut command_loop = CommandLoop::new();

    // Set temporary variables
    let mut command_exit = false;
    let mut write_confirm = false;
    let mut import = None;
    let mut execute = None;
    let mut schema = None;
    let mut command = None;

    for item in flags.iter() {
        match item.ftype {
            FlagType::Version => help::print_version(),
            FlagType::Help => help::print_binary_help_text(),
            FlagType::Confirm => write_confirm = true,
            FlagType::Argument => {
                if !item.option.is_empty() {
                    // If given file is .ced format execute it
                    if let Some(ext) = std::path::Path::new(&item.option).extension() {
                        if ext == "ced" {
                            execute.replace(item.option.clone());
                            command_exit = true;
                            continue;
                        }
                    }
                    import.replace(item.option.clone());
                }
            }
            FlagType::Schema => {
                if !item.option.is_empty() {
                    schema.replace(item.option.clone());
                } else {
                    utils::write_to_stderr("WRN : Schema is empty thus not applied\n")?;
                }
            }
            // Get stdin if given option is empty
            FlagType::Command => {
                if !item.option.is_empty() {
                    command.replace(item.option.clone());
                    command_exit = true;
                } else {
                    utils::write_to_stderr("WRN : Command is empty thus not executed\n")?;
                }
            }
            FlagType::NoLog => {
                command_loop.no_log();
            }
            FlagType::None => (),
        }

        if item.early_exit {
            return Ok(());
        }
    }

    // Add empty page
    command_loop.add_empty_page()?;

    // Add preset
    command_loop.processor.configure_preset(true)?;

    if let Some(execute) = execute.as_ref() {
        feed_execute(execute, &mut command_loop)?;
    }
    if let Some(import) = import.as_ref() {
        feed_import(import, &mut command_loop)?;
    }
    if let Some(sch) = schema.as_ref() {
        feed_schema(sch, &mut command_loop)?;
    }
    if let Some(cmd) = command.as_ref() {
        feed_command(cmd, &mut command_loop, write_confirm)?;
    }

    if command_exit {
        return Ok(());
    }
    // Handle error inside loop
    if let Some(err) = command_loop.start_loop().err() {
        println!("{}", err);
    }

    Ok(())
}

fn feed_execute(file: &str, command_loop: &mut CommandLoop) -> CedResult<()> {
    command_loop.toggle_no_loop(true);
    if let Err(err) =
        command_loop.feed_command(&Command::from_str(&format!("execute {}", file))?, true)
    {
        eprintln!("{}", err);
        command_loop.toggle_no_loop(false); // Return status, might be helpful
        return Ok(());
    }
    command_loop.toggle_no_loop(false);
    Ok(())
}

fn feed_import(file: &str, command_loop: &mut CommandLoop) -> CedResult<()> {
    if let Err(err) =
        command_loop.feed_command(&Command::from_str(&format!("import {}", file))?, true)
    {
        eprintln!("{}", err);
        return Ok(());
    }
    Ok(())
}

fn feed_schema(file: &str, command_loop: &mut CommandLoop) -> CedResult<()> {
    if let Err(err) =
        command_loop.feed_command(&Command::from_str(&format!("schema {} true", file))?, true)
    {
        eprintln!("{}", err);
        return Ok(());
    }
    Ok(())
}

fn feed_command(
    command: &str,
    command_loop: &mut CommandLoop,
    write_confirm: bool,
) -> CedResult<()> {
    let command_split: Vec<&str> = command.split_terminator(';').collect();
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

        if let Err(err) = command_loop.feed_command(&command, true) {
            eprintln!("{}", err);
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

    pub fn toggle_no_loop(&mut self, tv: bool) {
        self.processor.no_loop = tv;
    }

    pub fn no_log(&mut self) {
        self.processor.print_logs = false;
    }

    pub fn feed_command(&mut self, command: &Command, panic: bool) -> CedResult<()> {
        self.execute_command(command, panic)?;
        Ok(())
    }

    /// Start a loop until exit
    fn start_loop(&mut self) -> CedResult<()> {
        let mut command = Command::default();
        utils::write_to_stdout("Ced, a csv editor\n")?;
        let mut read_byte = 1;
        while read_byte != 0 && CommandType::Exit != command.command_type {
            utils::write_to_stdout(">> ")?;
            let mut input = String::new();
            read_byte = utils::read_stdin_until_eof(true, &mut input)?;
            if input.is_empty() {
                continue;
            }
            command = Command::from_str(&input)?;
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
            CommandType::Exit
            | CommandType::Import
            | CommandType::Export
            | CommandType::Create
            | CommandType::Write
            | CommandType::None(_)
            | CommandType::Schema
            | CommandType::SchemaInit
            | CommandType::SchemaExport
            | CommandType::PrintCell
            | CommandType::PrintRow
            | CommandType::PrintColumn
            | CommandType::Print => (),

            // Meta related
            CommandType::Help | CommandType::Version => (),

            _ => {
                let cursor = self
                    .processor
                    .get_cursor()
                    .ok_or(CedError::InvalidPageOperation("Page is empty".to_string()))?;
                self.history
                    .take_snapshot(self.processor.get_page_data(&cursor)?)
            }
        }

        if let Err(err) = self.processor.execute_command(command) {
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
            let cursor = self
                .processor
                .get_cursor()
                .ok_or(CedError::InvalidPageOperation("Page is empty".to_string()))?;
            *self.processor.get_page_data_mut(&cursor)? = prev.clone();
        }
        Ok(())
    }

    fn redo(&mut self) -> CedResult<()> {
        if let Some(prev) = self.history.pull_redo() {
            let cursor = self
                .processor
                .get_cursor()
                .ok_or(CedError::InvalidPageOperation("Page is empty".to_string()))?;
            *self.processor.get_page_data_mut(&cursor)? = prev;
        }
        Ok(())
    }

    fn add_empty_page(&mut self) -> CedResult<()> {
        self.processor.add_page("\\EMPTY", "", false, None, false)?;
        Ok(())
    }
}
