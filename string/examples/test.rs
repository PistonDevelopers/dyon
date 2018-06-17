extern crate dyon;
extern crate dyon_string;

use std::sync::Arc;
use dyon::*;

fn main() {
    let mut dyon_runtime = Runtime::new();
    let dyon_module = load_module().unwrap();
    if error(dyon_runtime.run(&Arc::new(dyon_module))) {
        return
    }
}

fn load_module() -> Option<dyon::Module> {
    let mut module = Module::new();
    dyon_string::add_functions(&mut module);
    if error(load("source/test.dyon", &mut module)) {
        None
    } else {
        Some(module)
    }
}
