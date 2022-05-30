/// Ced, a csv editing backend

#[cfg(test)]
mod test;

#[cfg(feature = "cli")]
pub(crate) mod cli;

pub(crate) mod command;
pub(crate) mod utils;

pub(crate) mod error;
pub(crate) mod processor;

// ----------
// RE-EXPORTS

#[cfg(feature = "cli")]
pub use cli::command_loop::start_main_loop;
pub use command::{Command, CommandType};
pub use error::{CedError, CedResult};
pub use processor::Processor;
