#[cfg(feature = "cli")]
use ced::{Command, CommandLoop, help, CedResult};
#[cfg(feature = "cli")]
use std::path::PathBuf;

#[cfg(feature = "cli")]
pub fn main() -> CedResult<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut import = None;

    // Print basic command line information
    for arg in &args[1..] {
        if arg.starts_with("--") || arg.starts_with("-") {
            if match_returning_flags(arg)? {
                return Ok(());
            }
        } else {
            import.replace(match_import(arg)?);
        }
    }

    // Start command loop
    let mut command_loop = CommandLoop::new();

    if let Some(file) = import {
        if let Err(err) = command_loop.feed_command(&Command::from_str(&format!("import {}", file.display()))?,true) {
            eprintln!("{}",err);
            return Ok(());
        }
    }

    // TODO
    //for arg in &args[1..] {
        //if arg.starts_with("--") || arg.starts_with("-") {
            //match_executing_flags(arg, &mut command_loop)?
        //} 
    //}

    command_loop
        .start_loop()
        .err()
        .map(|err| println!("{}", err));
    Ok(())
}

#[cfg(feature = "cli")]
/// Match arguments
fn match_import(arg: &str) -> CedResult<PathBuf> {
    Ok(PathBuf::from(arg))
}

/// Match flags
///
/// Return : if match should return early
#[cfg(feature = "cli")]
fn match_returning_flags(flag: &str) -> CedResult<bool> {
    match flag {
        "--version" | "-v" => {
            println!("ced, 0.1.3");
            return Ok(true);
        }
        "--help" | "-h" => {
            help::print_help_text();
            return Ok(true);
        }
        _ => (),
    }
    Ok(false)
}

// TODO
///// Match flags
/////
///// Return : if match should return early
//#[cfg(feature = "cli")]
//fn match_executing_flags(flag: &str, command_loop: &mut CommandLoop) -> CedResult<()> {
    //match flag {
        //"--version" | "-v" => {
            //println!("ced, 0.1.3");
            //return Ok(true);
        //}
        //_ => (),
    //}
    //Ok(false)
//}

// Placeholder for binary
#[cfg(not(feature = "cli"))]
pub fn main() {}
