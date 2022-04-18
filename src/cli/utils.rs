use crate::error::{CedError, CedResult};
use std::io::Write;

pub fn write_to_stdout(src: &str) -> CedResult<()> {
    write!(std::io::stdout(), "{}", src)
        .map_err(|err| CedError::io_error(err, "Failed to write to stdout"))?;
    std::io::stdout()
        .flush()
        .map_err(|err| CedError::io_error(err, "Failed to flush stdout"))?;

    Ok(())
}

pub fn write_to_stderr(src: &str) -> CedResult<()> {
    write!(std::io::stderr(), "{}", src)
        .map_err(|err| CedError::io_error(err, "Failed to write to stdout"))?;
    std::io::stderr()
        .flush()
        .map_err(|err| CedError::io_error(err, "Failed to flush stdout"))?;

    Ok(())
}

pub fn read_stdin(strip_newline: bool) -> CedResult<String> {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|err| CedError::io_error(err, "Failed to read stdin from source"))?;
    if strip_newline {
        input.pop();
    }
    Ok(input)
}
