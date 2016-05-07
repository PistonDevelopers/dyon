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

    let mut e: Option<Event> = None;
    let window_guard = CurrentGuard::new(&mut window);
    let event_guard: CurrentGuard<Option<Event>> = CurrentGuard::new(&mut e);
    if error(dyon_runtime.run(&dyon_module)) {
        return;
    }
    drop(event_guard);
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
    module.add(Arc::new("draw".into()), draw, PreludeFunction {
        lts: vec![Lt::Default],
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

mod dyon_functions {
    use dyon::Runtime;
    use current::Current;

    const NO_EVENT: &'static str = "No event";

    pub fn render(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        rt.push(unsafe { Current::<Option<Event>>::new()
            .as_ref().expect(NO_EVENT).render_args().is_some() });
        Ok(())
    }

    pub fn update(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        rt.push(unsafe { Current::<Option<Event>>::new()
            .as_ref().expect(NO_EVENT).update_args().is_some() });
        Ok(())
    }

    pub fn press(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        rt.push(unsafe { Current::<Option<Event>>::new()
            .as_ref().expect(NO_EVENT).press_args().is_some() });
        Ok(())
    }

    pub fn release(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        rt.push(unsafe { Current::<Option<Event>>::new()
            .as_ref().expect(NO_EVENT).release_args().is_some() });
        Ok(())
    }

    pub fn focus(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        rt.push(unsafe { Current::<Option<Event>>::new()
            .as_ref().expect(NO_EVENT).focus_args().is_some() });
        Ok(())
    }

    pub fn focus_arg(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        rt.push(unsafe { Current::<Option<Event>>::new()
            .as_ref().expect(NO_EVENT).focus_args() });
        Ok(())
    }

    pub fn draw(rt: &mut Runtime) -> Result<(), String> {
        use dyon::Variable;
        use std::sync::Arc;
        use piston_window::*;

        let window = unsafe { &mut *Current::<PistonWindow>::new() };
        let e = unsafe { &*Current::<Option<Event>>::new() };
        if let &Some(ref e) = e {
            let draw_list = rt.stack.pop().expect("There is no value on the stack");
            window.draw_2d(e, |c, g| {
                let arr = rt.resolve(&draw_list);
                if let &Variable::Array(ref arr) = arr {
                    for it in &**arr {
                        let it = rt.resolve(it);
                        if let &Variable::Array(ref it) = it {
                            let ty: Arc<String> = try!(rt.var(&it[0]));
                            match &**ty {

"clear" => {
    let color: [f32; 4] = try!(rt.var(&it[1]));
    clear(color, g);
}
"draw_color_radius_line" => {
    let color: [f32; 4] = try!(rt.var(&it[1]));
    let radius: f64 = try!(rt.var(&it[2]));
    let rect: [f64; 4] = try!(rt.var(&it[3]));
    line(color, radius, rect, c.transform, g);
}
"draw_color_rectangle" => {
    let color: [f32; 4] = try!(rt.var(&it[1]));
    let rect: [f64; 4] = try!(rt.var(&it[2]));
    rectangle(color, rect, c.transform, g);
}
"draw_color_ellipse" => {
    let color: [f32; 4] = try!(rt.var(&it[1]));
    let rect: [f64; 4] = try!(rt.var(&it[2]));
    ellipse(color, rect, c.transform, g);
}
"draw_color_ellipse_resolution" => {
    let color: [f32; 4] = try!(rt.var(&it[1]));
    let rect: [f64; 4] = try!(rt.var(&it[2]));
    let resolution: u32 = try!(rt.var(&it[3]));
    Ellipse::new(color)
        .resolution(resolution as u32)
        .draw(rect, &c.draw_state, c.transform, g);
}
_ => {}

                            }
                        }
                    }
                }
                return Ok(())
            }).unwrap_or(Ok(()))
        } else {
            Err(NO_EVENT.into())
        }
    }

    pub fn next_event(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let window = unsafe { &mut *Current::<PistonWindow>::new() };
        let e = unsafe { &mut *Current::<Option<Event>>::new() };
        if let Some(new_e) = window.next() {
            *e = Some(new_e);
            rt.push(true);
        } else {
            *e = None;
            rt.push(false);
        }
        Ok(())
    }

    pub fn set_title(rt: &mut Runtime) -> Result<(), String> {
        use std::sync::Arc;
        use piston_window::*;

        let window = unsafe { &mut *Current::<PistonWindow>::new() };
        let title: Arc<String> = try!(rt.pop());
        window.set_title((*title).clone());
        Ok(())
    }

    pub fn update_dt(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        rt.push(unsafe { Current::<Option<Event>>::new()
            .as_ref().expect(NO_EVENT).update_args().map(|args| args.dt) });
        Ok(())
    }

    pub fn press_keyboard_key(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<Option<Event>>::new() };
        if let &Some(ref e) = e {
            if let Some(Button::Keyboard(key)) = e.press_args() {
                rt.push(Some(key as u64 as f64));
            } else {
                rt.push::<Option<f64>>(None);
            }
            Ok(())
        } else {
            Err(NO_EVENT.into())
        }
    }

    pub fn release_keyboard_key(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<Option<Event>>::new() };
        if let &Some(ref e) = e {
            if let Some(Button::Keyboard(key)) = e.release_args() {
                rt.push(Some(key as u64 as f64));
            } else {
                rt.push::<Option<f64>>(None);
            }
            Ok(())
        } else {
            Err(NO_EVENT.into())
        }
    }
}
