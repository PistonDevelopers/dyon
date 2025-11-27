extern crate dyon;

use dyon::{error, run};

fn main() -> Result<(), ()> {
    let file = std::env::args_os()
        .nth(1)
        .and_then(|s| s.into_string().ok());
    if let Some(file) = file {
        if error(run(&file)) {Err(())} else {Ok(())}
    } else {
        eprintln!("dyonrun <file.dyon>");
        Err(())
    }
}
