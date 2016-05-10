extern crate glium_graphics;
extern crate piston;
extern crate dyon;
extern crate current;

use glium_graphics::{Glium2d, GliumWindow, OpenGL};
use piston::window::WindowSettings;
use piston::input::Event;
use dyon::{error, load, Module, Runtime};
use current::CurrentGuard;

mod helper;

fn main() {
    let opengl = OpenGL::V3_2;
    let ref mut window: GliumWindow = WindowSettings::new("Dyon example: Glium!", [512, 512])
        .opengl(opengl).exit_on_esc(true).build().unwrap();

    let mut runtime = Runtime::new();
    let module = match load_module() {
        None => return,
        Some(m) => m
    };

    let mut g2d = Glium2d::new(opengl, window);
    let mut e: Option<Event> = None;
    let window_guard = CurrentGuard::new(window);
    let event_guard: CurrentGuard<Option<Event>> = CurrentGuard::new(&mut e);
    let g2d_guard = CurrentGuard::new(&mut g2d);
    if error(runtime.run(&module)) {
        return;
    }
    drop(g2d_guard);
    drop(event_guard);
    drop(window_guard);
}


fn load_module() -> Option<Module> {
    use std::sync::Arc;
    use dyon_functions::*;
    use helper::add_functions;
    use dyon::{Lt, Module, PreludeFunction};
    use glium_graphics::GliumWindow;

    let mut module = Module::new();
    add_functions::<GliumWindow>(&mut module);
    module.add(Arc::new("draw".into()), draw, PreludeFunction {
        lts: vec![Lt::Default],
        returns: false
    });
    module.add(Arc::new("next_event".into()),
        next_event, PreludeFunction {
            lts: vec![],
            returns: true
        });
    if error(load("examples/piston_window/loader.rs", &mut module)) {
        None
    } else {
        Some(module)
    }
}

mod dyon_functions {
    use dyon::Runtime;
    use current::Current;
    use helper::{draw_2d, NO_EVENT};

    pub fn draw(rt: &mut Runtime) -> Result<(), String> {
        use piston::input::*;
        use glium_graphics::{Glium2d, GliumWindow};

        let window = unsafe { &mut *Current::<GliumWindow>::new() };
        let e = unsafe { &*Current::<Option<Event>>::new() };
        let g2d = unsafe { &mut *Current::<Glium2d>::new() };
        if let &Some(ref e) = e {
            if let Some(args) = e.render_args() {
                let mut target = window.draw();
                let res = g2d.draw(&mut target, args.viewport(), |c, g| {
                    draw_2d(rt, c, g)
                });
                target.finish().unwrap();
                res
            } else {
                Ok(())
            }
        } else {
            Err(NO_EVENT.into())
        }
    }

    pub fn next_event(rt: &mut Runtime) -> Result<(), String> {
        use piston::input::*;
        use glium_graphics::GliumWindow;

        let window = unsafe { &mut *Current::<GliumWindow>::new() };
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
}
