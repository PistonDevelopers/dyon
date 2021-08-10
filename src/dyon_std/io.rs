use std::error::Error;
use std::io;

/// Returns a string representation of an IO error.
pub fn io_error(action: &str, file: &str, err: &io::Error) -> String {
    format!(
        "IO Error when attempting to {} `{}`: {}\n{}",
        action,
        file,
        err.to_string(),
        match err.source() {
            None => "".to_string(),
            Some(cause) => cause.to_string(),
        }
    )
}
