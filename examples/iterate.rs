/*
> cargo build --release --example iterate --no-default-features
> time ./target/release/examples/iterate
*/

extern crate dyon;

use dyon::{error, run};

fn main() {
    error(run("source/bench/iterate.dyon"));
}
