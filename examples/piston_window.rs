extern crate dyon;
extern crate piston_window;
extern crate current;

use std::sync::Arc;
use piston_window::*;
use current::{Current, CurrentGuard};
use dyon::{error, intrinsics, load, Module, PreludeFunction, Runtime, Variable};

fn main() {
    let window: PistonWindow =
        WindowSettings::new("dyon: piston_window", [512; 2])
        .exit_on_esc(true)
        .samples(4)
        .build()
        .unwrap();
    let mut dyon_module = Module::new();
    let mut intrinsics = intrinsics::standard();
    intrinsics.insert(Arc::new("render".into()), PreludeFunction {
        arg_constraints: vec![],
        returns: true
    });
    if error(load("source/piston_window/square.rs", &mut dyon_module)) {
        return;
    }
    let mut dyon_runtime = Runtime::new();

    for mut e in window {
        let e_guard = CurrentGuard::new(&mut e);
        if error(dyon_runtime.run(&dyon_module)) {
            break;
        }
        drop(e_guard);
    }
}

fn render(_args: &[Variable]) -> Variable {
    let e = unsafe { &*Current::<PistonWindow>::new() };
    Variable::Bool(e.render_args().is_some())
}
