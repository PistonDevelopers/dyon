extern crate glium_graphics;
extern crate glium;
extern crate piston;
extern crate dyon;
extern crate current;
extern crate dyon_interactive;

use glium_graphics::{Glium2d, GliumWindow, OpenGL};
use piston::window::WindowSettings;
use piston::input::Event;
use dyon::{error, load, Module, Runtime};
use current::CurrentGuard;

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
    let mut target = window.draw();

    {
        let window_guard = CurrentGuard::new(window);
        let event_guard: CurrentGuard<Option<Event>> = CurrentGuard::new(&mut e);
        let g2d_guard = CurrentGuard::new(&mut g2d);
        let target_guard = CurrentGuard::new(&mut target);
        if error(runtime.run(&module)) {
            return;
        }
        drop(target_guard);
        drop(g2d_guard);
        drop(event_guard);
        drop(window_guard);
    }

    target.finish().unwrap();
}


fn load_module() -> Option<Module> {
    use std::sync::Arc;
    use dyon_functions::*;
    use dyon_interactive::add_functions;
    use dyon::{Lt, Module, Dfn, Type};
    use glium_graphics::GliumWindow;

    let mut module = Module::new();
    add_functions::<GliumWindow>(&mut module);
    module.add(Arc::new("draw".into()), draw, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::array()],
        ret: Type::Void
    });
    module.add(Arc::new("next_event".into()),
        next_event, Dfn {
            lts: vec![],
            tys: vec![],
            ret: Type::Bool
        });
    if error(load("examples/piston_window/loader.dyon", &mut module)) {
        None
    } else {
        Some(module)
    }
}

mod dyon_functions {
    use dyon::Runtime;
    use dyon_interactive::{draw_2d, NO_EVENT};
    use current::Current;

    pub fn draw(rt: &mut Runtime) -> Result<(), String> {
        use piston::input::*;
        use glium_graphics::Glium2d;
        use glium::Frame;

        let e = unsafe { &*Current::<Option<Event>>::new() };
        let g2d = unsafe { &mut *Current::<Glium2d>::new() };
        let target = unsafe { &mut *Current::<Frame>::new() };
        if let &Some(ref e) = e {
            if let Some(args) = e.render_args() {
                g2d.draw(target, args.viewport(), |c, g| {
                    draw_2d(rt, c, g)
                })
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
        use glium::Frame;

        let window = unsafe { &mut *Current::<GliumWindow>::new() };
        let e = unsafe { &mut *Current::<Option<Event>>::new() };
        let target = unsafe { &mut *Current::<Frame>::new() };
        if let Some(new_e) = window.next() {
            if new_e.after_render_args().is_some() {
                target.set_finish().unwrap();
                *target = window.draw();
            }
            *e = Some(new_e);
            rt.push(true);
        } else {
            *e = None;
            rt.push(false);
        }
        Ok(())
    }
}
