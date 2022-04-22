use crate::error::{CedError, CedResult};
use std::ffi::OsStr;
use std::io::Write;
use std::process::Stdio;

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

pub fn subprocess(args: &Vec<impl AsRef<OsStr>>, process_standard_input: Option<String>) -> CedResult<()> {
    let mut process = std::process::Command::new(&args[0])
        .args(&args[1..])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|_| {
            CedError::CliError(format!("Failed to execute command : \"{:?}\"", &args[0].as_ref()))
        })?;
    let mut stdin = process
        .stdin
        .take()
        .ok_or(CedError::CliError("Failed to read from stdin".to_string()))?;

    if let Some(input) = process_standard_input {
        std::thread::spawn(move || {
            stdin
                .write_all(input.as_bytes())
                .expect("Failed to write to stdin");
            });
    } 

    let output = process
        .wait_with_output()
        .map_err(|_| CedError::CliError("Failed to write to stdout".to_string()))?;
    let out_content = String::from_utf8_lossy(&output.stdout);
    let err_content = String::from_utf8_lossy(&output.stderr);

    if out_content.len() != 0 {
        write_to_stdout(&out_content)?;
    }
    if err_content.len() != 0 {
        write_to_stderr(&err_content)?;
    }
    Ok(())
}
