#[cfg(feature = "cli")]
use ced::CedResult;
#[cfg(feature = "cli")]
use std::path::PathBuf;

#[cfg(feature = "cli")]
pub fn main() -> CedResult<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut import = None;

    // Print basic command line information
    for arg in args {
        if arg.starts_with("--") || arg.starts_with("-") {
            if match_flags(&arg)? {
                return Ok(());
            }
        } else {
            import.replace(match_import(&arg)?);
        }
    }

    // Start command loop
    use ced::{Command, CommandLoop};
    let mut command_loop = CommandLoop::new();
    if let Some(file) = import {
        command_loop.feed_command(&Command::from_str(&format!("import {}", file.display()))?)?;
    }
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
fn match_flags(flag: &str) -> CedResult<bool> {
    match flag {
        "--version" | "-v" => {
            println!("ced, 0.1.2");
            return Ok(true);
        }
        "--help" | "-h" => {
            println!("{}", include_str!("../help.txt"));
            return Ok(true);
        }
        _ => (),
    }
    Ok(false)
}

// Placeholder for binary
#[cfg(not(feature = "cli"))]
pub fn main() {}
