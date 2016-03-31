extern crate dyon;
extern crate piston_window;
extern crate current;
extern crate timer_controller;

use std::sync::Arc;
use piston_window::*;
use current::{Current, CurrentGuard};
use dyon::{error, load, ArgConstraint, Module, PreludeFunction, Runtime, Variable};
use timer_controller::Timer;

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("dyon: piston_window", [512; 2])
        .exit_on_esc(true)
        .samples(4)
        .build()
        .unwrap();
    let mut dyon_module = match load_module() {
        None => return,
        Some(m) => m
    };
    let mut dyon_runtime = Runtime::new();

    let mut timer = Timer::new(0.25);
    let mut got_error = false;

    let window_guard = CurrentGuard::new(&mut window);
    if error(dyon_runtime.run(&dyon_module)) {
        return;
    }
    drop(window_guard);

    /*
    for mut e in window {
        timer.event(&e, || {
            if !got_error {
                dyon_module = match load_module() {
                    None => {
                        println!(" ~~~ Hit F1 to reload ~~~ ");
                        got_error = true;
                        return;
                    }
                    Some(m) => {
                        m
                    }
                };
            }
        });
        if let Some(Button::Keyboard(Key::F1)) = e.press_args() {
            println!(" ~~~ Reloading ~~~ ");
            got_error = false;
        }
        let e_guard = CurrentGuard::new(&mut e);
        if error(dyon_runtime.run(&dyon_module)) {
            break;
        }
        drop(e_guard);
    }
    */
}

fn load_module() -> Option<Module> {
    let mut module = Module::new();
    module.add(Arc::new("render".into()), dyon_render, PreludeFunction {
        arg_constraints: vec![],
        returns: true
    });
    module.add(Arc::new("clear".into()), dyon_clear, PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: false
    });
    module.add(Arc::new("draw_color_rect".into()),
        dyon_draw_color_rect, PreludeFunction {
            arg_constraints: vec![ArgConstraint::Default; 2],
            returns: false
        });
    module.add(Arc::new("next_event".into()),
        dyon_next_event, PreludeFunction {
            arg_constraints: vec![],
            returns: true
        });
    module.add(Arc::new("set_title".into()),
        dyon_set_title, PreludeFunction {
            arg_constraints: vec![ArgConstraint::Default],
            returns: false
        });
    if error(load("source/piston_window/square.rs", &mut module)) {
        None
    } else {
        Some(module)
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

fn pop_string(rt: &mut Runtime) -> Result<Arc<String>, String> {
    let v = rt.stack.pop().expect("Expected string");
    match rt.resolve(&v) {
        &Variable::Text(ref s) => Ok(s.clone()),
        _ => Err("Expected string".into())
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

fn dyon_draw_color_rect(rt: &mut Runtime) -> Result<(), String> {
    let e = unsafe { &mut *Current::<PistonWindow>::new() };
    let rect = try!(pop_rect(rt));
    let color = try!(pop_color(rt));
    e.draw_2d(|c, g| {
        rectangle(color, rect, c.transform, g);
    });
    Ok(())
}

fn dyon_next_event(rt: &mut Runtime) -> Result<(), String> {
    let e = unsafe { &mut *Current::<PistonWindow>::new() };
    if let Some(new_e) = e.next() {
        *e = new_e;
        push_bool(rt, true);
    } else {
        push_bool(rt, false);
    }
    Ok(())
}

fn dyon_set_title(rt: &mut Runtime) -> Result<(), String> {
    let e = unsafe { &mut *Current::<PistonWindow>::new() };
    let title = try!(pop_string(rt));
    e.set_title((*title).clone());
    Ok(())
}
