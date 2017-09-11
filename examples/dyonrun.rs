extern crate dyon;

use dyon::{error, run};

fn main() {
    let file = std::env::args_os().nth(1)
        .and_then(|s| s.into_string().ok());
    if let Some(file) = file {
        error(run(&file));
    } else {
        eprintln!("dyonrun <file.dyon>");
    }
}
