extern crate dyon;

use std::sync::Arc;
use dyon::*;

fn main() {
    let mut dyon_runtime = Runtime::new();
    let dyon_module = load_module().unwrap();
    if error(dyon_runtime.run(&dyon_module)) {
        return
    }
}

fn load_module() -> Option<Module> {
    let mut module = Module::new();
    module.add(Arc::new("say_hello".into()), dyon_say_hello, PreludeFunction {
        lts: vec![],
        returns: false
    });
    if error(load("source/functions/loader.rs", &mut module)) {
        None
    } else {
        Some(module)
    }
}

fn dyon_say_hello(_: &mut Runtime) -> Result<(), String> {
    println!("hi!");
    Ok(())
}
