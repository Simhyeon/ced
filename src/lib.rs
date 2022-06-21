/// Ced, a csv editing backend
///
/// ### Install
///
/// ```bash
/// cargo install ced --features cli --locked
/// ```
///
/// ### Binary usage
///
/// **Ced option**
///
/// ```bash
/// # Print version
/// ced --version
/// # Print help
/// ced --help
///
/// # Start ced
/// # Optionaly with initial import
/// ced
/// ced file.csv
///
/// # Execute script
/// # argument with .ced extension will be interpretted as execution script
/// # In this case, loop variants are restricted
/// ced script.ced
///
/// # Import schema and import data file.
/// # Execute a given command without opening an interactive shell
/// ced --schema schema.csv data.csv --command 'add-row 1 100,20;write'
/// ```
///
/// **Ced shell command**
///
/// ```bash
/// # Type help in prompt or give --help flag for detailed usage.
///
/// # Get help
/// >> help
///
/// # Import a file
/// >> import file_name.csv
///
/// # Import a schema file. Second argument determines overriding.
/// >> schema file_name true
///
/// # Print csv data optionally with a viewer command
/// # Set CED_VIEWER to set default print viewer
/// >> print
/// >> print tidy-viwer
///
/// # Append a new row to last
/// # Type a comma to exit loop
/// >> add-row
/// First Header = .. <USER_INPUT>
/// Second Header = .. <USER_INPUT>
///
/// # Edit a given row
/// >> edit-row <ROW_NUMBER>
///
/// # Set a limiter for a column with interactive shell
/// >> limit
///
/// # Export to a file
/// >> export file_name.csv
///
/// # Overwrite to a source file
/// >> write
///
/// # Undo a previous operation
/// # History capacity is 16 by default
/// # You can override it with CED_HISTORY_CAPACITY
/// >> undo
///
/// # Redo a previous undo
/// >> redo
/// ```

#[cfg(test)]
mod test;

#[cfg(feature = "cli")]
pub(crate) mod cli;

pub(crate) mod command;
pub(crate) mod utils;

pub(crate) mod error;
pub(crate) mod page;
pub(crate) mod processor;

// ----------
// RE-EXPORTS

#[cfg(feature = "cli")]
pub use cli::command_loop::start_main_loop;
pub use command::{Command, CommandType};
pub use error::{CedError, CedResult};
pub use processor::Processor;
