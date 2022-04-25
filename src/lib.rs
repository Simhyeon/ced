/// Ced, a csv editing backend

#[cfg(test)]
mod test;

#[cfg(feature = "cli")]
pub(crate) mod cli;

pub(crate) mod command;
pub(crate) mod utils;

pub(crate) mod error;
pub(crate) mod processor;
pub(crate) mod value;
pub(crate) mod virtual_data;

// ----------
// RE-EXPORTS

#[cfg(feature = "cli")]
pub use cli::command_loop::start_main_loop;
pub use error::{CedError, CedResult};
pub use processor::Processor;
pub use value::{Value, ValueLimiter, ValueType};
pub use command::{Command, CommandType};
