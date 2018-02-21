#[macro_use]
extern crate dyon;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate piston;
extern crate current;
extern crate dyon_interactive;
extern crate music;
extern crate image;

use std::sync::Arc;
use current::CurrentGuard;
use dyon::{error, load, Lt, Module, Dfn, Runtime, Type};
use dyon_interactive::{FontNames, ImageNames};
use image::RgbaImage;
use piston::input::Event;
use piston::window::WindowSettings;
use piston::event_loop::{Events, EventSettings};
use sdl2_window::Sdl2Window;
use opengl_graphics::{OpenGL, Filter, GlGraphics, GlyphCache, Texture, TextureSettings};

#[derive(Clone, Hash, PartialEq, Eq)]
enum Music {
    Name(Arc<String>),
}

#[derive(Clone, Hash, PartialEq, Eq)]
enum Sound {
    Name(Arc<String>),
}

fn main() {
    let file = std::env::args_os().nth(1)
        .and_then(|s| s.into_string().ok());
    let file = if let Some(file) = file {
        use std::env::set_current_dir;
        use std::path::PathBuf;

        let path: PathBuf = (&file).into();
        if let Some(parent) = path.parent() {
            if let Err(_) = set_current_dir(parent) {
                file
            } else {
                path.file_name().unwrap().to_str().unwrap().to_owned()
            }
        } else {
            file
        }
    } else {
        println!("dyongame <file.dyon>");
        return;
    };

    let opengl = OpenGL::V3_2;
    let mut window: Sdl2Window =
        WindowSettings::new("dyongame", [512; 2])
        .exit_on_esc(true)
        .samples(4)
        .opengl(opengl)
        .build()
        .unwrap();
    let dyon_module = match load_module(&file) {
        None => return,
        Some(m) => Arc::new(m)
    };

    let mut factory = ();
    let mut dyon_runtime = Runtime::new();
    let fira_sans = include_bytes!("../assets/FiraSans-Regular.ttf");
    let hack = include_bytes!("../assets/Hack-Regular.ttf");
    let font_texture_settings = TextureSettings::new().filter(Filter::Nearest);
    let mut glyphs = vec![
        GlyphCache::from_bytes(&fira_sans[..], (), font_texture_settings.clone()).unwrap(),
        GlyphCache::from_bytes(&hack[..], (), font_texture_settings).unwrap()
    ];
    let mut font_names = FontNames(vec![
        Arc::new("FiraSans-Regular".to_owned()),
        Arc::new("Hack-Regular".to_owned()),
    ]);
    let mut images = vec![];
    let mut image_names = ImageNames(vec![]);
    let mut textures = vec![];
    let mut gl = GlGraphics::new(opengl);
    let mut events = Events::new(EventSettings::new());

    let mut e: Option<Event> = None;
    let sdl = window.sdl_context.clone();
    let factory_guard: CurrentGuard<()> = CurrentGuard::new(&mut factory);
    let window_guard = CurrentGuard::new(&mut window);
    let event_guard: CurrentGuard<Option<Event>> = CurrentGuard::new(&mut e);
    let glyphs_guard: CurrentGuard<Vec<GlyphCache>> = CurrentGuard::new(&mut glyphs);
    let font_names_guard: CurrentGuard<FontNames> = CurrentGuard::new(&mut font_names);
    let images_guard: CurrentGuard<Vec<RgbaImage>> = CurrentGuard::new(&mut images);
    let image_names_guard: CurrentGuard<ImageNames> = CurrentGuard::new(&mut image_names);
    let textures_guard: CurrentGuard<Vec<Texture>> = CurrentGuard::new(&mut textures);
    let gl_guard: CurrentGuard<GlGraphics> = CurrentGuard::new(&mut gl);
    let events_guard: CurrentGuard<Events> = CurrentGuard::new(&mut events);

    music::start_context::<Music, Sound, _>(&sdl, 16, || {
        if error(dyon_runtime.run(&dyon_module)) {
            return;
        }
    });

    drop(events_guard);
    drop(gl_guard);
    drop(textures_guard);
    drop(image_names_guard);
    drop(images_guard);
    drop(font_names_guard);
    drop(glyphs_guard);
    drop(event_guard);
    drop(window_guard);
    drop(factory_guard);
}

fn load_module(file: &str) -> Option<Module> {
    use dyon_functions::*;
    use dyon_interactive::add_functions;

    let mut module = Module::new();
    add_functions::<Sdl2Window, (), GlyphCache>(&mut module);
    module.add(Arc::new("render_source".into()), render_source, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::Text
    });
    module.add(Arc::new("draw".into()), draw, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::array()],
        ret: Type::Void
    });
    module.add(Arc::new("set_window__size".into()), set_window__size, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::Vec4],
        ret: Type::Void
    });
    module.add(Arc::new("next_event".into()),
        next_event, Dfn {
            lts: vec![],
            tys: vec![],
            ret: Type::Bool
        });
    module.add(Arc::new("bind_sound__name_file".into()),
        bind_sound__name_file, Dfn {
            lts: vec![Lt::Default; 2],
            tys: vec![Type::Text; 2],
            ret: Type::Void
        });
    module.add(Arc::new("bind_music__name_file".into()),
        bind_music__name_file, Dfn {
            lts: vec![Lt::Default; 2],
            tys: vec![Type::Text; 2],
            ret: Type::Void
        });
    module.add(Arc::new("play_sound__name_repeat_volume".into()),
        play_sound__name_repeat_volume, Dfn {
            lts: vec![Lt::Default; 3],
            tys: vec![Type::Text, Type::F64, Type::F64],
            ret: Type::Void
        });
    module.add(Arc::new("play_sound_forever__name_volume".into()),
        play_sound_forever__name_volume, Dfn {
            lts: vec![Lt::Default; 2],
            tys: vec![Type::Text, Type::F64],
            ret: Type::Void
        });
    module.add(Arc::new("play_music__name_repeat".into()),
        play_music__name_repeat, Dfn {
            lts: vec![Lt::Default; 2],
            tys: vec![Type::Text, Type::F64],
            ret: Type::Void
        });
    module.add(Arc::new("play_music_forever__name".into()),
        play_music_forever__name, Dfn {
            lts: vec![Lt::Default; 1],
            tys: vec![Type::Text],
            ret: Type::Void
        });
    module.add(Arc::new("set_music_volume".into()),
        set_music_volume, Dfn {
            lts: vec![Lt::Default],
            tys: vec![Type::F64],
            ret: Type::Void
        });
    module.add(Arc::new("create_texture".into()),
        create_texture, Dfn {
            lts: vec![Lt::Default],
            tys: vec![Type::F64],
            ret: Type::F64,
        }
    );
    module.add(Arc::new("update__texture_image".into()),
        update__texture_image, Dfn {
            lts: vec![Lt::Default; 2],
            tys: vec![Type::F64, Type::F64],
            ret: Type::Void
        }
    );

    if error(dyon::load_str(
        "render.dyon",
        Arc::new(include_str!("../src/render.dyon").into()),
        &mut module
    )) {
        return None;
    }

    if error(load(file, &mut module)) {
        None
    } else {
        Some(module)
    }
}

mod dyon_functions {
    use sdl2_window::Sdl2Window;
    use piston::input::{Event, RenderEvent};
    use piston::event_loop::Events;
    use opengl_graphics::{GlGraphics, GlyphCache, Texture, TextureSettings};
    use dyon::Runtime;
    use dyon_interactive::{draw_2d, NO_EVENT};
    use current::Current;
    use std::sync::Arc;
    use music;
    use {Music, Sound};
    use image::RgbaImage;

    dyon_fn!{fn render_source() -> String {include_str!("../src/render.dyon").into()}}

    pub fn draw(rt: &mut Runtime) -> Result<(), String> {
        let e = unsafe { &*Current::<Option<Event>>::new() };
        let gl = unsafe { &mut *Current::<GlGraphics>::new() };
        let glyphs = unsafe { &mut *Current::<Vec<GlyphCache>>::new() };
        let textures = unsafe { &mut *Current::<Vec<Texture>>::new() };
        if let &Some(ref e) = e {
            if let Some(args) = e.render_args() {
                gl.draw(args.viewport(), |c, g| {
                    draw_2d(rt, glyphs, textures, c, g)
                })
            } else {
                Ok(())
            }
        } else {
            Err(NO_EVENT.into())
        }
    }

    pub fn create_texture(rt: &mut Runtime) -> Result<(), String> {
        let images = unsafe { &*Current::<Vec<RgbaImage>>::new() };
        let textures = unsafe { &mut *Current::<Vec<Texture>>::new() };
        let id: usize = rt.pop()?;
        let new_id = textures.len();
        let image = if let Some(x) = images.get(id) {
            x
        } else {
            return Err("Image id is out of bounds".into());
        };
        textures.push(Texture::from_image(image, &TextureSettings::new()));
        rt.push(new_id);
        Ok(())
    }

    #[allow(non_snake_case)]
    pub fn update__texture_image(rt: &mut Runtime) -> Result<(), String> {
        let images = unsafe { &*Current::<Vec<RgbaImage>>::new() };
        let textures = unsafe { &mut *Current::<Vec<Texture>>::new() };
        let image_id: usize = rt.pop()?;
        let texture_id: usize = rt.pop()?;
        let image = if let Some(x) = images.get(image_id) {
            x
        } else {
            return Err("Image id is out of bounds".into());
        };
        let texture = if let Some(x) = textures.get_mut(texture_id) {
            x
        } else {
            return Err("Texture id is out of bounds".into());
        };
        texture.update(&image);
        Ok(())
    }

    #[allow(non_snake_case)]
    pub fn set_window__size(rt: &mut Runtime) -> Result<(), String> {
        let window = unsafe { &mut *Current::<Sdl2Window>::new() };
        let size: [f32; 2] = rt.pop_vec4()?;
        let _ = window.window.set_size(size[0] as u32, size[1] as u32);
        Ok(())
    }

    pub fn next_event(rt: &mut Runtime) -> Result<(), String> {
        let window = unsafe { &mut *Current::<Sdl2Window>::new() };
        let events = unsafe { &mut *Current::<Events>::new() };
        let e = unsafe { &mut *Current::<Option<Event>>::new() };
        if let Some(new_e) = events.next(window) {
            *e = Some(new_e);
            rt.push(true);
        } else {
            *e = None;
            rt.push(false);
        }
        Ok(())
    }

    #[allow(non_snake_case)]
    pub fn bind_sound__name_file(rt: &mut Runtime) -> Result<(), String> {
        let file: Arc<String> = rt.pop()?;
        let name: Arc<String> = rt.pop()?;
        music::bind_sound_file(Sound::Name(name), &**file);
        Ok(())
    }

    #[allow(non_snake_case)]
    pub fn bind_music__name_file(rt: &mut Runtime) -> Result<(), String> {
        let file: Arc<String> = rt.pop()?;
        let name: Arc<String> = rt.pop()?;
        music::bind_music_file(Music::Name(name), &**file);
        Ok(())
    }

    #[allow(non_snake_case)]
    pub fn play_sound__name_repeat_volume(rt: &mut Runtime) -> Result<(), String> {
        use music::Repeat;

        let volume: f64 = rt.pop()?;
        let repeat: f64 = rt.pop()?;
        let name: Arc<String> = rt.pop()?;
        let repeat = if repeat == -1.0 {Repeat::Forever} else {Repeat::Times(repeat as u16)};
        music::play_sound(&Sound::Name(name), repeat, volume);
        Ok(())
    }

    #[allow(non_snake_case)]
    pub fn play_sound_forever__name_volume(rt: &mut Runtime) -> Result<(), String> {
        use music::Repeat;

        let volume: f64 = rt.pop()?;
        let name: Arc<String> = rt.pop()?;
        music::play_sound(&Sound::Name(name), Repeat::Forever, volume);
        Ok(())
    }

    #[allow(non_snake_case)]
    pub fn play_music__name_repeat(rt: &mut Runtime) -> Result<(), String> {
        use music::Repeat;

        let repeat: f64 = rt.pop()?;
        let name: Arc<String> = rt.pop()?;
        let repeat = if repeat == -1.0 {Repeat::Forever} else {Repeat::Times(repeat as u16)};
        music::play_music(&Music::Name(name), repeat);
        Ok(())
    }

    #[allow(non_snake_case)]
    pub fn play_music_forever__name(rt: &mut Runtime) -> Result<(), String> {
        use music::Repeat;

        let name: Arc<String> = rt.pop()?;
        music::play_music(&Music::Name(name), Repeat::Forever);
        Ok(())
    }

    pub fn set_music_volume(rt: &mut Runtime) -> Result<(), String> {
        let volume: f64 = rt.pop()?;
        music::set_volume(volume);
        Ok(())
    }
}
