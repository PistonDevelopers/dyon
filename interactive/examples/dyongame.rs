extern crate dyon;
extern crate piston_window;
extern crate current;
extern crate dyon_interactive;

use std::sync::Arc;
use piston_window::*;
use current::CurrentGuard;
use dyon::{error, load, Lt, Module, Dfn, Runtime, Type};

fn main() {
    let file = std::env::args_os().nth(1)
        .and_then(|s| s.into_string().ok());
    let file = if let Some(file) = file {
        use std::env::set_current_dir;
        use std::path::PathBuf;

        let path: PathBuf = (&file).into();
        if let Some(parent) = path.parent() {
            set_current_dir(parent).expect("Could not set current directory");
            path.file_name().unwrap().to_str().unwrap().to_owned()
        } else {
            file
        }
    } else {
        println!("dyongame <file.dyon>");
        return;
    };

    let mut window: PistonWindow =
        WindowSettings::new("dyongame", [512; 2])
        .exit_on_esc(true)
        .samples(4)
        .build()
        .unwrap();
    let dyon_module = match load_module(&file) {
        None => return,
        Some(m) => Arc::new(m)
    };
    let mut dyon_runtime = Runtime::new();
    let factory = window.factory.clone();
    let font = "../../assets/FiraSans-Regular.ttf";
    let mut glyphs = Glyphs::new(font, factory, TextureSettings::new()).unwrap();

    let mut e: Option<Event> = None;
    let window_guard = CurrentGuard::new(&mut window);
    let event_guard: CurrentGuard<Option<Event>> = CurrentGuard::new(&mut e);
    let glyphs_guard: CurrentGuard<Glyphs> = CurrentGuard::new(&mut glyphs);
    if error(dyon_runtime.run(&dyon_module)) {
        return;
    }
    drop(glyphs_guard);
    drop(event_guard);
    drop(window_guard);
}

fn load_module(file: &str) -> Option<Module> {
    use dyon_functions::*;
    use dyon_interactive::add_functions;

    let mut module = Module::new();
    add_functions::<PistonWindow, Glyphs>(&mut module);
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
    if error(load(file, &mut module)) {
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
        use piston_window::*;

        let window = unsafe { &mut *Current::<PistonWindow>::new() };
        let e = unsafe { &*Current::<Option<Event>>::new() };
        let glyphs = unsafe { &mut *Current::<Glyphs>::new() };
        if let &Some(ref e) = e {
            window.draw_2d(e, |c, g| {
                draw_2d(rt, glyphs, c, g)
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
}
