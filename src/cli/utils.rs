use crate::error::{CedResult, CedError};
use std::io::Write;

pub(crate) fn write_to_stdout(src: &str) -> CedResult<()> {
    write!(std::io::stdout(), "{}", src)
        .map_err(|err| {
            CedError::io_error(err, "Failed to write to stdout")
        })?;
    std::io::stdout().flush()
        .map_err(|err| {
            CedError::io_error(err, "Failed to flush stdout")
        })?;
        
    Ok(())
}

pub(crate) fn write_to_stderr(src: &str) -> CedResult<()> {
    write!(std::io::stderr(), "{}", src)
        .map_err(|err| {
            CedError::io_error(err, "Failed to write to stdout")
        })?;
    std::io::stderr().flush()
        .map_err(|err| {
            CedError::io_error(err, "Failed to flush stdout")
        })?;
        
    Ok(())
}

pub(crate) fn read_stdin() -> CedResult<String> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)
        .map_err(|err| {
            CedError::io_error(err, "Failed to read stdin from source")
        })?;
    Ok(input)
}
