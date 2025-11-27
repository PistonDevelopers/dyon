#[macro_use]
extern crate dyon;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate piston;
extern crate current;
extern crate dyon_interactive;
extern crate kira;
extern crate image;
extern crate graphics;

use std::sync::Arc;
use std::collections::HashMap;
use current::CurrentGuard;
use dyon::{error, load, Module, Dfn, Runtime, Type};
use dyon_interactive::{FontNames, ImageNames};
use image::RgbaImage;
use piston::input::Event;
use piston::window::WindowSettings;
use piston::event_loop::{Events, EventSettings};
use sdl2_window::Sdl2Window;
use opengl_graphics::{OpenGL, Filter, GlGraphics, GlyphCache, Texture, TextureSettings};
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::handle::SoundHandle;
use kira::mixer::{SubTrackHandle, SubTrackSettings};

type Sounds = HashMap<Arc<String>, SoundHandle>;
type Music = HashMap<Arc<String>, SoundHandle>;

fn main() -> Result<(), ()> {
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
        return Err(());
    };

    let opengl = OpenGL::V3_2;
    let mut window: Sdl2Window =
        WindowSettings::new("dyongame", [512; 2])
        .exit_on_esc(true)
        .samples(4)
        .graphics_api(opengl)
        .build()
        .unwrap();
    let dyon_module = match load_module(&file) {
        None => return Err(()),
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
    let mut audio_manager = AudioManager::new(AudioManagerSettings::default()).unwrap();
    let mut music_track = audio_manager.add_sub_track(SubTrackSettings::default()).unwrap();
    let mut sounds: Sounds = HashMap::new();
    let mut music: Music = HashMap::new();

    let mut e: Option<Event> = None;
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
    let audio_manager_guard: CurrentGuard<AudioManager> = CurrentGuard::new(&mut audio_manager);
    let music_track_guard: CurrentGuard<SubTrackHandle> = CurrentGuard::new(&mut music_track);
    let sounds_guard: CurrentGuard<Sounds> = CurrentGuard::new(&mut sounds);
    let music_guard: CurrentGuard<Music> = CurrentGuard::new(&mut music);

    if error(dyon_runtime.run(&dyon_module)) {
        return Err(());
    }

    drop(music_guard);
    drop(sounds_guard);
    drop(music_track_guard);
    drop(audio_manager_guard);
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

    Ok(())
}

fn load_module(file: &str) -> Option<Module> {
    use dyon_functions::*;
    use dyon_interactive::add_functions;

    let mut module = Module::new();
    add_functions::<Sdl2Window, (), GlyphCache>(&mut module);
    module.add(Arc::new("render_source".into()), render_source, Dfn::nl(vec![], Type::Str));
    module.add(Arc::new("draw".into()), draw, Dfn::nl(vec![Type::array()], Type::Void));
    module.add(Arc::new("next_event".into()),
        next_event, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("bind_sound__name_file".into()),
        bind_sound__name_file, Dfn::nl(vec![Type::Str; 2], Type::Void));
    module.add(Arc::new("bind_music__name_file".into()),
        bind_music__name_file, Dfn::nl(vec![Type::Str; 2], Type::Void));
    module.add(Arc::new("play_sound__name_repeat_volume".into()),
        play_sound__name_repeat_volume, Dfn::nl(vec![Type::Str, Type::F64, Type::F64], Type::Void));
    module.add(Arc::new("play_sound_forever__name_volume".into()),
        play_sound_forever__name_volume, Dfn::nl(vec![Type::Str, Type::F64], Type::Void));
    module.add(Arc::new("play_music__name_repeat".into()),
        play_music__name_repeat, Dfn::nl(vec![Type::Str, Type::F64], Type::Void));
    module.add(Arc::new("play_music_forever__name".into()),
        play_music_forever__name, Dfn::nl(vec![Type::Str], Type::Void));
    module.add(Arc::new("set_music_volume".into()),
        set_music_volume, Dfn::nl(vec![Type::F64], Type::Void));
    module.add(Arc::new("create_texture".into()),
        create_texture, Dfn {
            lts: vec![dyon::Lt::Default],
            tys: vec![Type::Any],
            ret: Type::Any,
            ext: vec![(vec![], vec![Type::F64, Type::F64], Type::F64)],
            lazy: dyon::LAZY_NO,
        });
    module.add(Arc::new("update__texture_image".into()),
        update__texture_image, Dfn::nl(vec![Type::F64, Type::F64], Type::Void)
    );
    module.add(Arc::new("load_font".into()),
        load_font, Dfn::nl(vec![Type::Str], Type::Result(Box::new(Type::F64)))
    );
    module.add(Arc::new("load_font_obj".into()),
        load_font_obj, Dfn::nl(vec![Type::Str], Type::Result(Box::new(Type::Any)))
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
    use dyon::{Runtime, Variable};
    use dyon_interactive::{draw_2d, FontNames, NO_EVENT};
    use current::Current;
    use std::sync::Arc;
    use image::RgbaImage;

    dyon_fn!{fn render_source() -> String {include_str!("../src/render.dyon").into()}}


    pub fn load_font(rt: &mut Runtime) -> Result<Variable, String> {
        use dyon::embed::PushVariable;
        use opengl_graphics::{Filter, TextureSettings};
        use graphics::glyph_cache::rusttype::GlyphCache;

        let glyphs = unsafe { &mut *Current::<Vec<GlyphCache<'static, (), Texture>>>::new() };
        let font_names = unsafe { &mut *Current::<FontNames>::new() };
        let file: Arc<String> = rt.pop()?;
        let texture_settings = TextureSettings::new().filter(Filter::Nearest);
        Ok(match GlyphCache::<'static, (), Texture>::new(&**file, (), texture_settings) {
            Ok(x) => {
                let id = glyphs.len();
                glyphs.push(x);
                font_names.0.push(file.clone());
                Ok::<usize, Arc<String>>(id).push_var()
            }
            Err(err) => {
                Err::<usize, Arc<String>>(Arc::new(format!("{}", err))).push_var()
            }
        })
    }

    pub fn load_font_obj(rt: &mut Runtime) -> Result<Variable, String> {
        use dyon::embed::{to_rust_object, PushVariable};
        use dyon::RustObject;
        use opengl_graphics::{Filter, TextureSettings};
        use graphics::glyph_cache::rusttype::GlyphCache;

        let file: Arc<String> = rt.pop()?;
        let texture_settings = TextureSettings::new().filter(Filter::Nearest);
        Ok(match GlyphCache::<'static, (), Texture>::new(
            &**file, (), texture_settings
        ) {
            Ok(x) => Ok::<RustObject, Arc<String>>(to_rust_object(x)).push_var(),
            Err(err) => Err::<RustObject, Arc<String>>(Arc::new(format!("{}", err))).push_var(),
        })
    }

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

    pub fn create_texture(rt: &mut Runtime) -> Result<Variable, String> {
        use dyon::embed::{to_rust_object, PushVariable};

        let image: Variable = rt.pop()?;
        match image {
            Variable::F64(id, _) => {
                let images = unsafe { &*Current::<Vec<RgbaImage>>::new() };
                let textures = unsafe { &mut *Current::<Vec<Texture>>::new() };
                let image: &RgbaImage = if let Some(x) = images.get(id as usize) {x}
                            else {return Err("Image id is out of bounds".into())};

                let new_id = textures.len();
                textures.push(Texture::from_image(image, &TextureSettings::new()));
                Ok(new_id.push_var())
            }
            Variable::RustObject(obj) => {
                let mut guard = obj.lock().map_err(|_| "Could not obtain lock on Mutex".to_string())?;
                let image: &mut RgbaImage = guard.downcast_mut()
                    .ok_or_else(|| "Expected RgbaImage".to_string())?;

                Ok(Variable::RustObject(
                    to_rust_object(Texture::from_image(image, &TextureSettings::new()))))
            }
            _ => Err("Expected `f64` (image id) or `any` (rust object)".to_string()),
        }
    }

    #[allow(non_snake_case)]
    pub fn update__texture_image(rt: &mut Runtime) -> Result<(), String> {
        let image: Variable = rt.pop()?;
        let texture: Variable = rt.pop()?;
        match image {
            Variable::F64(id, _) => {
                let images = unsafe { &*Current::<Vec<RgbaImage>>::new() };
                let image: &RgbaImage = if let Some(x) = images.get(id as usize) {x}
                            else {return Err("Image id is out of bounds".into())};
                match texture {
                    Variable::F64(id, _) => {
                        let textures = unsafe { &mut *Current::<Vec<Texture>>::new() };
                        let texture: &mut Texture = if let Some(x) = textures.get_mut(id as usize) {x}
                                    else {return Err("Texture id is out of bounds".into())};
                        texture.update(&image);
                        Ok(())
                    }
                    Variable::RustObject(obj) => {
                        let mut guard = obj.lock()
                            .map_err(|_| "Could not obtain lock on Mutex".to_string())?;
                        let texture: &mut Texture = guard.downcast_mut()
                            .ok_or_else(|| "Expected Texture".to_string())?;
                        texture.update(&image);
                        Ok(())
                    }
                    _ => Err("Expected `f64` (texture id) or `any` (rust object)".to_string()),
                }
            }
            Variable::RustObject(obj) => {
                let mut guard = obj.lock().map_err(|_| "Could not obtain lock on Mutex".to_string())?;
                let image: &mut RgbaImage = guard.downcast_mut()
                    .ok_or_else(|| "Expected RgbaImage".to_string())?;
                match texture {
                    Variable::F64(id, _) => {
                        let textures = unsafe { &mut *Current::<Vec<Texture>>::new() };
                        let texture: &mut Texture = if let Some(x) = textures.get_mut(id as usize) {x}
                                    else {return Err("Texture id is out of bounds".into())};
                        texture.update(&image);
                        Ok(())
                    }
                    Variable::RustObject(obj) => {
                        let mut guard = obj.lock()
                            .map_err(|_| "Could not obtain lock on Mutex".to_string())?;
                        let texture: &mut Texture = guard.downcast_mut()
                            .ok_or_else(|| "Expected Texture".to_string())?;
                        texture.update(&image);
                        Ok(())
                    }
                    _ => Err("Expected `f64` (texture id) or `any` (rust object)".to_string()),
                }
            }
            _ => Err("Expected `f64` (image id) or `any` (rust object)".to_string()),
        }
    }

    dyon_fn!{fn next_event() -> bool {
        let window = unsafe { &mut *Current::<Sdl2Window>::new() };
        let events = unsafe { &mut *Current::<Events>::new() };
        let e = unsafe { &mut *Current::<Option<Event>>::new() };
        if let Some(new_e) = events.next(window) {
            *e = Some(new_e);
            true
        } else {
            *e = None;
            false
        }
    }}

    dyon_fn!{fn bind_sound__name_file(name: Arc<String>, file: Arc<String>) {
        use kira::sound::SoundSettings;
        use crate::AudioManager;
        use crate::Sounds;

        let audio_manager = unsafe { &mut *Current::<AudioManager>::new() };
        let sounds = unsafe { &mut *Current::<Sounds>::new() };
        let sound_handle = audio_manager.load_sound(&**file, SoundSettings::default()).unwrap();
        sounds.insert(name, sound_handle);
    }}

    dyon_fn!{fn bind_music__name_file(name: Arc<String>, file: Arc<String>) {
        use kira::sound::SoundSettings;
        use crate::AudioManager;
        use crate::Music;

        let audio_manager = unsafe { &mut *Current::<AudioManager>::new() };
        let music = unsafe { &mut *Current::<Music>::new() };
        let sound_handle = audio_manager.load_sound(&**file, SoundSettings::default()).unwrap();
        music.insert(name, sound_handle);
    }}

    dyon_fn!{fn play_sound__name_repeat_volume(name: Arc<String>, repeat: f64, volume: f64) {
        use kira::instance::InstanceSettings;
        use kira::arrangement::{
            Arrangement,
            ArrangementSettings,
            LoopArrangementSettings,
            SoundClip
        };
        use crate::Sounds;
        use crate::AudioManager;

        let audio_manager = unsafe { &mut *Current::<AudioManager>::new() };
        let sounds = unsafe { &mut *Current::<Sounds>::new() };
        if let Some(sound_handle) = sounds.get_mut(&name) {
            let instance_settings = InstanceSettings::default().volume(volume);
            if repeat == -1.0 {
                let mut arrangement_handle = audio_manager.add_arrangement(Arrangement::new_loop(
                	&sound_handle,
                	LoopArrangementSettings::default(),
                )).unwrap();
                arrangement_handle.play(instance_settings).unwrap();
            } else if repeat != 0.0 {
                let mut arrangement = Arrangement::new(ArrangementSettings::new());
                let mut start = 0.0;
                for _ in 0..repeat as u32 {
                    arrangement.add_clip(SoundClip::new(&sound_handle, start));
                    start += sound_handle.duration();
                }
                let mut arrangement_handle = audio_manager.add_arrangement(arrangement).unwrap();
                arrangement_handle.play(instance_settings).unwrap();
            }
        }
    }}

    dyon_fn!{fn play_sound_forever__name_volume(name: Arc<String>, volume: f64) {
        use kira::instance::InstanceSettings;
        use kira::arrangement::{
            Arrangement,
            LoopArrangementSettings,
        };
        use crate::Sounds;
        use crate::AudioManager;

        let audio_manager = unsafe { &mut *Current::<AudioManager>::new() };
        let sounds = unsafe { &mut *Current::<Sounds>::new() };
        if let Some(sound_handle) = sounds.get_mut(&name) {
            let instance_settings = InstanceSettings::default().volume(volume);
            let mut arrangement_handle = audio_manager.add_arrangement(Arrangement::new_loop(
            	&sound_handle,
            	LoopArrangementSettings::default(),
            )).unwrap();
            arrangement_handle.play(instance_settings).unwrap();
        }
    }}

    dyon_fn!{fn play_music__name_repeat(name: Arc<String>, repeat: f64) {
        use kira::instance::InstanceSettings;
        use kira::arrangement::{
            Arrangement,
            ArrangementSettings,
            LoopArrangementSettings,
            SoundClip
        };
        use crate::Music;
        use crate::AudioManager;
        use crate::SubTrackHandle;

        let audio_manager = unsafe { &mut *Current::<AudioManager>::new() };
        let music = unsafe { &mut *Current::<Music>::new() };
        let music_track = unsafe { &mut *Current::<SubTrackHandle>::new() };
        if let Some(sound_handle) = music.get_mut(&name) {
            let instance_settings = InstanceSettings::default();
            if repeat == -1.0 {
                let mut arrangement_handle = audio_manager.add_arrangement(Arrangement::new_loop(
                	&sound_handle,
                	LoopArrangementSettings::default().default_track(music_track.id()),
                )).unwrap();
                arrangement_handle.play(instance_settings).unwrap();
            } else if repeat != 0.0 {
                let mut arrangement = Arrangement::new(ArrangementSettings::new()
                    .default_track(music_track.id()));
                let mut start = 0.0;
                for _ in 0..repeat as u32 {
                    arrangement.add_clip(SoundClip::new(&sound_handle, start));
                    start += sound_handle.duration();
                }
                let mut arrangement_handle = audio_manager.add_arrangement(arrangement).unwrap();
                arrangement_handle.play(instance_settings).unwrap();
            }
        }
    }}

    dyon_fn!{fn play_music_forever__name(name: Arc<String>) {
        use crate::kira::instance::InstanceSettings;
        use crate::kira::arrangement::{
            Arrangement,
            LoopArrangementSettings,
        };
        use crate::Music;
        use crate::AudioManager;
        use crate::SubTrackHandle;

        let audio_manager = unsafe { &mut *Current::<AudioManager>::new() };
        let music = unsafe { &mut *Current::<Music>::new() };
        let music_track = unsafe { &mut *Current::<SubTrackHandle>::new() };
        if let Some(sound_handle) = music.get_mut(&name) {
            let instance_settings = InstanceSettings::default();
            let mut arrangement_handle = audio_manager.add_arrangement(Arrangement::new_loop(
            	&sound_handle,
            	LoopArrangementSettings::default().default_track(music_track.id()),
            )).unwrap();
            arrangement_handle.play(instance_settings).unwrap();
        }
    }}

    dyon_fn!{fn set_music_volume(volume: f64) {
        use crate::SubTrackHandle;

        let music_track = unsafe { &mut *Current::<SubTrackHandle>::new() };
        music_track.set_volume(volume).unwrap();
    }}
}
