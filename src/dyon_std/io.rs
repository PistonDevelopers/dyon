use std::io;
use std::error::Error;

/// Returns a string representation of an IO error.
pub fn io_error(action: &str, file: &str, err: &io::Error) -> String {
    format!("IO Error when attempting to {} `{}`: {}\n{}", action, file, err.description(),
        match err.cause() {
            None => "",
            Some(cause) => cause.description()
        })
}
