extern crate dyon;

use std::sync::Arc;
use dyon::{run_str, error};

fn main() {
    error(run_str("main.dyon", Arc::new(r#"
        fn main() {
            println("Hi!")
            println("1 + 1 = " + str(1+1))
        }
    "#.into())));
}
