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
    module.add(Arc::new("draw_color_rectangle".into()),
        draw_color_rectangle, PreludeFunction {
            lts: vec![Lt::Default; 2],
            returns: false
        });
    module.add(Arc::new("draw_color_ellipse".into()),
        draw_color_ellipse, PreludeFunction {
            lts: vec![Lt::Default; 2],
            returns: false
        });
    module.add(Arc::new("draw_color_ellipse_resolution".into()),
        draw_color_ellipse_resolution, PreludeFunction {
            lts: vec![Lt::Default; 3],
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

mod dyon_functions {
    use dyon::Runtime;
    use current::Current;

    pub fn render(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<PistonWindow>::new() };
        rt.push(e.render_args().is_some());
        Ok(())
    }

    pub fn update(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<PistonWindow>::new() };
        rt.push(e.update_args().is_some());
        Ok(())
    }

    pub fn press(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<PistonWindow>::new() };
        rt.push(e.press_args().is_some());
        Ok(())
    }

    pub fn release(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<PistonWindow>::new() };
        rt.push(e.release_args().is_some());
        Ok(())
    }

    pub fn focus(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<PistonWindow>::new() };
        rt.push(e.focus_args().is_some());
        Ok(())
    }

    pub fn focus_arg(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &*Current::<PistonWindow>::new() };
        rt.push(e.focus_args());
        Ok(())
    }

    pub fn clear(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        let color: [f32; 4] = try!(rt.pop());
        e.draw_2d(|_c, g| {
            clear(color, g);
        });
        Ok(())
    }

    pub fn draw_color_rectangle(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        let rect: [f64; 4] = try!(rt.pop());
        let color: [f32; 4] = try!(rt.pop());
        e.draw_2d(|c, g| {
            rectangle(color, rect, c.transform, g);
        });
        Ok(())
    }

    pub fn draw_color_ellipse(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        let rect: [f64; 4] = try!(rt.pop());
        let color: [f32; 4] = try!(rt.pop());
        e.draw_2d(|c, g| {
            ellipse(color, rect, c.transform, g);
        });
        Ok(())
    }

    pub fn draw_color_ellipse_resolution(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };

        let resolution: f64 = try!(rt.pop());
        let rect: [f64; 4] = try!(rt.pop());
        let color: [f32; 4] = try!(rt.pop());
        e.draw_2d(|c, g| {
            Ellipse::new(color)
                .resolution(resolution as u32)
                .draw(rect, &c.draw_state, c.transform, g);
        });
        Ok(())
    }

    pub fn draw_color_radius_line(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        let rect: [f64; 4] = try!(rt.pop());
        let radius: f64 = try!(rt.pop());
        let color: [f32; 4] = try!(rt.pop());
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
            rt.push(true);
        } else {
            rt.push(false);
        }
        Ok(())
    }

    pub fn set_title(rt: &mut Runtime) -> Result<(), String> {
        use std::sync::Arc;
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        let title: Arc<String> = try!(rt.pop());
        e.set_title((*title).clone());
        Ok(())
    }

    pub fn update_dt(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        rt.push(e.update_args().map(|args| args.dt));
        Ok(())
    }

    pub fn press_keyboard_key(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        if let Some(Button::Keyboard(key)) = e.press_args() {
            rt.push(Some(key as u64 as f64));
        } else {
            rt.push::<Option<f64>>(None);
        }
        Ok(())
    }

    pub fn release_keyboard_key(rt: &mut Runtime) -> Result<(), String> {
        use piston_window::*;

        let e = unsafe { &mut *Current::<PistonWindow>::new() };
        if let Some(Button::Keyboard(key)) = e.release_args() {
            rt.push(Some(key as u64 as f64));
        } else {
            rt.push::<Option<f64>>(None);
        }
        Ok(())
    }
}
