use crate::error::{CedError, CedResult};
use std::ffi::OsStr;
use std::io::Write;
use std::process::Stdio;

pub(crate) const DEFAULT_DELIMITER: &str = ",";

#[allow(unused_variables)]
pub fn write_to_stdout(src: &str) -> CedResult<()> {
    #[cfg(not(test))]
    write!(std::io::stdout(), "{}", src)
        .map_err(|err| CedError::io_error(err, "Failed to write to stdout"))?;
    std::io::stdout()
        .flush()
        .map_err(|err| CedError::io_error(err, "Failed to flush stdout"))?;

    Ok(())
}

#[allow(unused_variables)]
pub(crate) fn write_to_stderr(src: &str) -> CedResult<()> {
    #[cfg(not(test))]
    write!(std::io::stderr(), "{}", src)
        .map_err(|err| CedError::io_error(err, "Failed to write to stdout"))?;
    std::io::stderr()
        .flush()
        .map_err(|err| CedError::io_error(err, "Failed to flush stdout"))?;

    Ok(())
}

#[cfg(feature = "cli")]
pub(crate) fn read_stdin_until_eof(strip_newline: bool, input: &mut String) -> CedResult<usize> {
    let read_byte = std::io::stdin()
        .read_line(input)
        .map_err(|err| CedError::io_error(err, "Failed to read stdin from source"))?;
    if strip_newline && (input.ends_with('\n') | input.ends_with("\r\n")) {
        *input = input.trim().to_owned();
    }
    Ok(read_byte)
}

pub(crate) fn read_stdin(strip_newline: bool) -> CedResult<String> {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|err| CedError::io_error(err, "Failed to read stdin from source"))?;
    if strip_newline && (input.ends_with('\n') || input.ends_with("\r\n")) {
        input = input.trim().to_owned();
    }
    Ok(input)
}

pub(crate) fn subprocess(
    args: &[impl AsRef<OsStr>],
    process_standard_input: Option<String>,
) -> CedResult<()> {
    #[cfg(target_os = "windows")]
    let mut process = std::process::Command::new("cmd")
        .arg("/C")
        .args(&args[0..])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|_| {
            CedError::CommandError(format!(
                "Failed to execute command : \"{:?}\"",
                &args[0].as_ref()
            ))
        })?;
    #[cfg(not(target_os = "windows"))]
    let mut process = std::process::Command::new("sh")
        .arg("-c")
        .args(&args[0..])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|_| {
            CedError::CommandError(format!(
                "Failed to execute command : \"{:?}\"",
                &args[0].as_ref()
            ))
        })?;

    let mut stdin = process
        .stdin
        .take()
        .ok_or_else(|| CedError::CommandError("Failed to read from stdin".to_string()))?;

    if let Some(input) = process_standard_input {
        std::thread::spawn(move || {
            stdin
                .write_all(input.as_bytes())
                .expect("Failed to write to stdin");
        });
    }

    let output = process
        .wait_with_output()
        .map_err(|_| CedError::CommandError("Failed to write to stdout".to_string()))?;
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

/// Check if given string has valid csv spec
///
/// This will return None if given value doesn't qualify with csv spec
pub(crate) fn is_valid_csv(value: &str) -> bool {
    let mut on_quote = false;
    let mut previous = ' ';
    let mut iter = value.chars().peekable();
    while let Some(ch) = iter.next() {
        match ch {
            '"' => {
                // Add literal double quote if previous was same character
                if previous == '"' {
                    previous = ' '; // Reset previous
                } else {
                    // Start quote
                    if let Some('"') = iter.peek() {
                    } else {
                        on_quote = !on_quote;
                    }
                    previous = ch;
                }
            }
            ',' => {
                // This is unallowed in csv spec
                if !on_quote {
                    return false;
                } else {
                }
            }
            _ => previous = ch,
        }
    }

    // If quote is not finished, then it is invalid
    !on_quote
}

pub fn tokens_with_quote(source: &str) -> Vec<String> {
    let mut tokens = vec![];
    let mut on_quote = false;
    let mut previous = ' ';
    let mut chunk = String::new();
    let iter = source.chars().peekable();
    for ch in iter {
        match ch {
            // Escape character should not bed added
            '\\' => {
                if previous == '\\' {
                    previous = ' '; // Reset previous
                } else {
                    on_quote = !on_quote;
                    previous = ch;
                    continue;
                }
            }
            '\'' => {
                // Add literal quote if previous was escape character
                if previous == '\\' {
                    previous = ' '; // Reset previous
                } else {
                    on_quote = !on_quote;
                    previous = ch;
                    continue;
                }
            }
            ' ' => {
                if !on_quote {
                    // If previous is also blank. skip
                    if previous == ' ' {
                        continue;
                    }
                    let flushed = std::mem::take(&mut chunk);
                    tokens.push(flushed);
                    previous = ch;
                    continue;
                }
            }
            _ => previous = ch,
        }
        chunk.push(ch);
    }
    if !chunk.is_empty() {
        tokens.push(chunk);
    }
    tokens
}
