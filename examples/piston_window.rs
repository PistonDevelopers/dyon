extern crate dyon;
extern crate piston_window;
extern crate current;

use std::sync::Arc;
use piston_window::*;
use current::CurrentGuard;
use dyon::{error, load, Lt, Module, PreludeFunction, Runtime};

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("dyon: piston_window", [512; 2])
        .exit_on_esc(true)
        .samples(4)
        .build()
        .unwrap();
    let dyon_module = match load_module() {
        None => return,
        Some(m) => m
    };
    let mut dyon_runtime = Runtime::new();

    let window_guard = CurrentGuard::new(&mut window);
    if error(dyon_runtime.run(&dyon_module)) {
        return;
    }
    drop(window_guard);
}

fn load_module() -> Option<Module> {
    use dyon_functions::*;

    let mut module = Module::new();
    module.add(Arc::new("render".into()), render, PreludeFunction {
        lts: vec![],
        returns: true
    });
    module.add(Arc::new("update".into()), update, PreludeFunction {
        lts: vec![],
        returns: true
    });
    module.add(Arc::new("press".into()), press, PreludeFunction {
        lts: vec![],
        returns: true
    });
    module.add(Arc::new("release".into()), release, PreludeFunction {
        lts: vec![],
        returns: true
    });
    module.add(Arc::new("focus".into()), focus, PreludeFunction {
        lts: vec![],
        returns: true,
    });
    module.add(Arc::new("focus_arg".into()), focus_arg, PreludeFunction {
        lts: vec![],
        returns: true,
    });
    module.add(Arc::new("clear".into()), clear, PreludeFunction {
        lts: vec![Lt::Default],
        returns: false
    });
    module.add(Arc::new("draw_color_rect".into()),
        draw_color_rect, PreludeFunction {
            lts: vec![Lt::Default; 2],
            returns: false
        });
    module.add(Arc::new("draw_color_radius_line".into()),
        draw_color_radius_line, PreludeFunction {
            lts: vec![Lt::Default; 3],
            returns: false
        });
    module.add(Arc::new("next_event".into()),
        next_event, PreludeFunction {
            lts: vec![],
            returns: true
        });
    module.add(Arc::new("set_title".into()),
        set_title, PreludeFunction {
            lts: vec![Lt::Default],
            returns: false
        });
    module.add(Arc::new("update_dt".into()),
        update_dt, PreludeFunction {
            lts: vec![],
            returns: true
        });
    module.add(Arc::new("press_keyboard_key".into()),
        press_keyboard_key, PreludeFunction {
            lts: vec![],
            returns: true
        });
    module.add(Arc::new("release_keyboard_key".into()),
        release_keyboard_key, PreludeFunction {
            lts: vec![],
            returns: true
        });
    if error(load("source/piston_window/loader.rs", &mut module)) {
        None
    } else {
        Some(module)
    }
}

mod stack {
    use dyon::{Runtime, Variable};
    use std::sync::Arc;

    pub fn push_opt_bool(rt: &mut Runtime, val: Option<bool>) {
        match val {
            None => {
                rt.stack.push(Variable::Option(None))
            }
            Some(b) => {
                rt.stack.push(Variable::Option(Some(Box::new(Variable::Bool(b)))))
            }
        }
    }

    pub fn push_bool(rt: &mut Runtime, val: bool) {
        rt.stack.push(Variable::Bool(val))
    }

    pub fn push_opt_num(rt: &mut Runtime, val: Option<f64>) {
        match val {
            None => {
                rt.stack.push(Variable::Option(None))
            }
            Some(n) => {
                rt.stack.push(Variable::Option(Some(Box::new(Variable::F64(n)))))
            }
        }
    }

    pub fn pop_num(rt: &mut Runtime) -> Result<f64, String> {
        let num = rt.stack.pop().expect("Expected number");
        match rt.resolve(&num) {
            &Variable::F64(n) => Ok(n),
            _ => Err("Expected number".into())
        }
    }

    pub fn pop_color(rt: &mut Runtime) -> Result<[f32; 4], String> {
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

    pub fn pop_rect(rt: &mut Runtime) -> Result<[f64; 4], String> {
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

    pub fn pop_string(rt: &mut Runtime) -> Result<Arc<String>, String> {
        let v = rt.stack.pop().expect("Expected string");
        match rt.resolve(&v) {
            &Variable::Text(ref s) => Ok(s.clone()),
            _ => Err("Expected string".into())
        }
    }
}

mod dyon_functions {
    use dyon::Runtime;
    use current::Current;
    use stack::*;

    pub fn render(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<PistonWindow>::new() };
        push_bool(rt, e.render_args().is_some());
        Ok(())
    }

    pub fn update(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<PistonWindow>::new() };
        push_bool(rt, e.update_args().is_some());
        Ok(())
    }

    pub fn press(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<PistonWindow>::new() };
        push_bool(rt, e.press_args().is_some());
        Ok(())
    }

    pub fn release(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<PistonWindow>::new() };
        push_bool(rt, e.release_args().is_some());
        Ok(())
    }

    pub fn focus(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<PistonWindow>::new() };
        push_bool(rt, e.focus_args().is_some());
        Ok(())
    }

    pub fn focus_arg(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<PistonWindow>::new() };
        push_opt_bool(rt, e.focus_args());
        Ok(())
    }

    pub fn clear(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        let color = try!(pop_color(rt));
        e.draw_2d(|_c, g| {
            clear(color, g);
        });
        Ok(())
    }

    pub fn draw_color_rect(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        let rect = try!(pop_rect(rt));
        let color = try!(pop_color(rt));
        e.draw_2d(|c, g| {
            rectangle(color, rect, c.transform, g);
        });
        Ok(())
    }

    pub fn draw_color_radius_line(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        let rect = try!(pop_rect(rt));
        let radius = try!(pop_num(rt));
        let color = try!(pop_color(rt));
        e.draw_2d(|c, g| {
            line(color, radius, rect, c.transform, g);
        });
        Ok(())
    }

    pub fn next_event(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        if let Some(new_e) = e.next() {
            *e = new_e;
            push_bool(rt, true);
        } else {
            push_bool(rt, false);
        }
        Ok(())
    }

    pub fn set_title(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        let title = try!(pop_string(rt));
        e.set_title((*title).clone());
        Ok(())
    }

    pub fn update_dt(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        push_opt_num(rt, e.update_args().map(|args| args.dt));
        Ok(())
    }

    pub fn press_keyboard_key(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        if let Some(Button::Keyboard(key)) = e.press_args() {
            push_opt_num(rt, Some(key as u64 as f64));
        } else {
            push_opt_num(rt, None);
        }
        Ok(())
    }

    pub fn release_keyboard_key(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        if let Some(Button::Keyboard(key)) = e.release_args() {
            push_opt_num(rt, Some(key as u64 as f64));
        } else {
            push_opt_num(rt, None);
        }
        Ok(())
    }
}
