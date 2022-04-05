/// Ced, a csv editing backend

#[cfg(feature = "cli")]
pub(crate) mod cli;

pub(crate) mod error;
pub mod processor;
pub(crate) mod value;
pub(crate) mod virtual_data;

#[cfg(feature = "cli")]
pub use cli::{CommandLoop, Command};
pub use error::{CedError, CedResult};
pub use processor::Processor;
pub use value::{Value, ValueLimiter, ValueType};
