extern crate dyon;

use dyon::{error, run_str};
use std::sync::Arc;

fn main() {
    error(run_str(
        "main.dyon",
        Arc::new(
            r#"
        fn main() {
            println("Hi!")
            println("1 + 1 = " + str(1+1))
        }
    "#
            .into(),
        ),
    ));
}
