extern crate dyon;
extern crate piston_window;
extern crate current;

use std::sync::Arc;
use piston_window::*;
use current::{Current, CurrentGuard};
use dyon::{error, load, ArgConstraint, Module, PreludeFunction, Runtime, Variable};

fn main() {
    let window: PistonWindow =
        WindowSettings::new("dyon: piston_window", [512; 2])
        .exit_on_esc(true)
        .samples(4)
        .build()
        .unwrap();
    let mut dyon_module = Module::new();
    dyon_module.add(Arc::new("render".into()), dyon_render, PreludeFunction {
        arg_constraints: vec![],
        returns: true
    });
    dyon_module.add(Arc::new("clear".into()), dyon_clear, PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: false
    });
    dyon_module.add(Arc::new("rectangle_color_rect".into()),
        dyon_rectangle_color_rect, PreludeFunction {
            arg_constraints: vec![ArgConstraint::Default; 2],
            returns: false
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

fn dyon_render(rt: &mut Runtime) -> Result<(), String> {
    let e = unsafe { &*Current::<PistonWindow>::new() };
    push_bool(rt, e.render_args().is_some());
    Ok(())
}

fn push_bool(rt: &mut Runtime, val: bool) {
    rt.stack.push(Variable::Bool(val))
}

fn pop_color(rt: &mut Runtime) -> Result<[f32; 4], String> {
    let color = rt.stack.pop().expect("Expected color");
    match rt.resolve(&color) {
        &Variable::Array(ref arr) => {
            let r = match rt.resolve(&arr[0]) {
                &Variable::F64(r) => r,
                _ => return Err("Expected number".into())
            };
            let g = match rt.resolve(&arr[1]) {
                &Variable::F64(r) => r,
                _ => return Err("Expected number".into())
            };
            let b = match rt.resolve(&arr[2]) {
                &Variable::F64(r) => r,
                _ => return Err("Expected number".into())
            };
            let a = match rt.resolve(&arr[3]) {
                &Variable::F64(r) => r,
                _ => return Err("Expected number".into())
            };
            Ok([r as f32, g as f32, b as f32, a as f32])
        }
        _ => return Err("Expected color".into())
    }
}

fn pop_rect(rt: &mut Runtime) -> Result<[f64; 4], String> {
    let v = rt.stack.pop().expect("Expected rect");
    match rt.resolve(&v) {
        &Variable::Array(ref arr) => {
            let x = match rt.resolve(&arr[0]) {
                &Variable::F64(x) => x,
                _ => return Err("Expected number".into())
            };
            let y = match rt.resolve(&arr[1]) {
                &Variable::F64(y) => y,
                _ => return Err("Expected number".into())
            };
            let w = match rt.resolve(&arr[2]) {
                &Variable::F64(w) => w,
                _ => return Err("Expected number".into())
            };
            let h = match rt.resolve(&arr[3]) {
                &Variable::F64(h) => h,
                _ => return Err("Expected number".into())
            };
            Ok([x, y, w, h])
        }
        _ => return Err("Expected rect".into())
    }
}

fn dyon_clear(rt: &mut Runtime) -> Result<(), String> {
    let e = unsafe { &mut *Current::<PistonWindow>::new() };
    let color = try!(pop_color(rt));
    e.draw_2d(|_c, g| {
        clear(color, g);
    });
    Ok(())
}

fn dyon_rectangle_color_rect(rt: &mut Runtime) -> Result<(), String> {
    let e = unsafe { &mut *Current::<PistonWindow>::new() };
    let rect = try!(pop_rect(rt));
    let color = try!(pop_color(rt));
    e.draw_2d(|c, g| {
        rectangle(color, rect, c.transform, g);
    });
    Ok(())
}
