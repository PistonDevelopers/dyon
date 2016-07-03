extern crate dyon;

use dyon::{error, run_string};

fn main() {
    error(run_string("fn main() { println(\"Goodbye, world!\") }".into()));
}
