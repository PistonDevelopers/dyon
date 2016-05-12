extern crate dyon;

use dyon::{error, run};

fn main() {
    error(run("source/test.dyon"));
}
