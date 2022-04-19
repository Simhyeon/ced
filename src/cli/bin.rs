#[cfg(feature = "cli")]
use ced::{Command, CommandLoop, help, CedResult, Parser, FlagType, CommandType, utils};

#[cfg(feature = "cli")]
pub fn main() -> CedResult<()> {
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

#[cfg(feature = "cli")]
fn feed_import(file: &str, command_loop: &mut CommandLoop) -> CedResult<()> {
    if let Err(err) = command_loop.feed_command(&Command::from_str(&format!("import {}", file))?,true) {
        eprintln!("{}",err);
        return Ok(());
    }
    Ok(())
}

#[cfg(feature = "cli")]
fn feed_schema(file: &str, command_loop: &mut CommandLoop) -> CedResult<()> {
    if let Err(err) = command_loop.feed_command(&Command::from_str(&format!("schema {} true", file))?,true) {
        eprintln!("{}",err);
        return Ok(());
    }
    Ok(())
}

#[cfg(feature = "cli")]
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

// Placeholder for binary
#[cfg(not(feature = "cli"))]
pub fn main() {}
