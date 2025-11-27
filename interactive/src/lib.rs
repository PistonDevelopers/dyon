extern crate piston;
extern crate dyon;
extern crate current;
extern crate graphics;
extern crate texture;
extern crate image;

use std::any::Any;
use std::sync::Arc;
use self::dyon::*;
use self::current::Current;
use self::piston::input::*;
use self::piston::window::*;
use self::graphics::{Context, Graphics};
use self::graphics::character::CharacterCache;
use texture::{CreateTexture, UpdateTexture};

pub const NO_EVENT: &'static str = "No event";

/// Adds functions to module, using a generic backend.
///
/// `W` is window.
/// `F` is factory (to create textures).
/// `C` is character cache.
pub fn add_functions<W, F, C>(module: &mut Module)
    where W: Any + AdvancedWindow,
          F: 'static + Clone,
          C::Texture: CreateTexture<F> + UpdateTexture<F>,
          C: Any + CharacterCache,
{
    module.add(Arc::new("window_size".into()), window_size::<W>,
        Dfn::nl(vec![], Type::Vec4));
    module.add(Arc::new("set_window__size".into()), set_window__size::<W>,
        Dfn::nl(vec![Type::Vec4], Type::Void));
    module.add(Arc::new("window_draw_size".into()), window_draw_size::<W>,
        Dfn::nl(vec![], Type::Vec4));
    module.add(Arc::new("window_position".into()), window_position::<W>,
        Dfn::nl(vec![], Type::Vec4));
    module.add(Arc::new("set_window__position".into()), set_window__position::<W>,
        Dfn::nl(vec![Type::Vec4], Type::Void));
    module.add(Arc::new("render".into()), render, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("after_render".into()), after_render, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("update".into()), update, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("idle".into()), idle, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("press".into()), press, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("release".into()), release, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("resize".into()), resize, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("focus".into()), focus, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("cursor".into()), cursor, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("text".into()), text, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("mouse_cursor".into()), mouse_cursor,
        Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("focus_arg".into()), focus_arg,
        Dfn::nl(vec![], Type::Option(Box::new(Type::Bool))));
    module.add(Arc::new("cursor_arg".into()), cursor_arg,
        Dfn::nl(vec![], Type::Option(Box::new(Type::Bool))));
    module.add(Arc::new("mouse_cursor_pos".into()), mouse_cursor_pos,
        Dfn::nl(vec![], Type::Option(Box::new(Type::Vec4))));
    module.add(Arc::new("window_title".into()),
        window_title::<W>, Dfn::nl(vec![], Type::Str));
    module.add(Arc::new("set_window__title".into()),
        set_window__title::<W>, Dfn::nl(vec![Type::Str], Type::Void));
    module.add(Arc::new("event_loop_ups".into()),
        event_loop_ups, Dfn::nl(vec![], Type::F64));
    module.add(Arc::new("set_event_loop__ups".into()),
        set_event_loop__ups, Dfn::nl(vec![Type::F64], Type::Void));
    module.add(Arc::new("event_loop_upsreset".into()),
        event_loop_upsreset, Dfn::nl(vec![], Type::F64));
    module.add(Arc::new("set_event_loop__upsreset".into()),
        set_event_loop__upsreset, Dfn::nl(vec![Type::F64], Type::Void));
    module.add(Arc::new("event_loop_maxfps".into()),
        event_loop_maxfps, Dfn::nl(vec![], Type::F64));
    module.add(Arc::new("set_event_loop__maxfps".into()),
        set_event_loop__maxfps, Dfn::nl(vec![Type::F64], Type::Void));
    module.add(Arc::new("event_loop_swapbuffers".into()),
        event_loop_swapbuffers, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("set_event_loop__swapbuffers".into()),
        set_event_loop__swapbuffers, Dfn::nl(vec![Type::Bool], Type::Void));
    module.add(Arc::new("swap_buffers".into()),
        swap_buffers::<W>, Dfn::nl(vec![], Type::Void));
    module.add(Arc::new("event_loop_benchmode".into()),
        event_loop_benchmode, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("set_event_loop__benchmode".into()),
        set_event_loop__benchmode, Dfn::nl(vec![Type::Bool], Type::Void));
    module.add(Arc::new("event_loop_lazy".into()),
        event_loop_lazy, Dfn::nl(vec![], Type::Bool));
    module.add(Arc::new("set_event_loop__lazy".into()),
        set_event_loop__lazy, Dfn::nl(vec![Type::Bool], Type::Void));
    module.add(Arc::new("render_ext_dt".into()),
        render_ext_dt, Dfn::nl(vec![], Type::Option(Box::new(Type::F64))));
    module.add(Arc::new("update_dt".into()),
        update_dt, Dfn::nl(vec![], Type::Option(Box::new(Type::F64))));
    module.add(Arc::new("idle_dt".into()),
        idle_dt, Dfn::nl(vec![], Type::Option(Box::new(Type::F64))));
    module.add(Arc::new("press_keyboard_key".into()),
        press_keyboard_key, Dfn::nl(vec![], Type::Option(Box::new(Type::F64))));
    module.add(Arc::new("release_keyboard_key".into()),
        release_keyboard_key, Dfn::nl(vec![], Type::Option(Box::new(Type::F64))));
    module.add(Arc::new("press_mouse_button".into()),
        press_mouse_button, Dfn::nl(vec![], Type::Option(Box::new(Type::F64))));
    module.add(Arc::new("release_mouse_button".into()),
        release_mouse_button, Dfn::nl(vec![], Type::Option(Box::new(Type::F64))));
    module.add(Arc::new("text_arg".into()),
        text_arg, Dfn::nl(vec![], Type::Option(Box::new(Type::Str))));
    module.add(Arc::new("width__font_size_string".into()),
        width__font_size_string::<C>, Dfn::nl(vec![Type::F64, Type::F64, Type::Str], Type::F64));
    module.add(Arc::new("font_names".into()),
        font_names, Dfn::nl(vec![], Type::Array(Box::new(Type::Str)))
    );
    module.add(Arc::new("load_font".into()),
        load_font::<F, C::Texture>, Dfn::nl(vec![Type::Str], Type::Result(Box::new(Type::F64)))
    );
    module.add(Arc::new("image_names".into()),
        image_names, Dfn::nl(vec![], Type::Array(Box::new(Type::Str)))
    );
    module.add(Arc::new("load_image".into()),
        load_image, Dfn::nl(vec![Type::Str], Type::Result(Box::new(Type::F64)))
    );
    module.add(Arc::new("create_image__name_size".into()),
        create_image__name_size, Dfn::nl(vec![Type::Str, Type::Vec4], Type::F64)
    );
    module.add(Arc::new("save__image_file".into()),
        save__image_file, Dfn::nl(vec![Type::F64, Type::Str], Type::Result(Box::new(Type::Str)))
    );
    module.add(Arc::new("image_size".into()),
        image_size, Dfn::nl(vec![Type::F64], Type::Vec4)
    );
    module.add(Arc::new("pxl__image_pos_color".into()),
        pxl__image_pos_color, Dfn::nl(vec![Type::F64, Type::Vec4, Type::Vec4], Type::Void)
    );
    module.add(Arc::new("pxl__image_pos".into()),
        pxl__image_pos, Dfn::nl(vec![Type::F64, Type::Vec4], Type::Vec4)
    );
}

pub fn window_size<W: Any + Window>(_rt: &mut Runtime) -> Result<Variable, String> {
    let size = unsafe { Current::<W>::new() }.size();
    Ok(Variable::Vec4([size.width as f32, size.height as f32, 0.0, 0.0]))
}

pub fn window_draw_size<W: Any + Window>(_rt: &mut Runtime) -> Result<Variable, String> {
    let draw_size = unsafe { Current::<W>::new() }.draw_size();
    Ok(Variable::Vec4([draw_size.width as f32, draw_size.height as f32, 0.0, 0.0]))
}

#[allow(non_snake_case)]
pub fn set_window__size<W: Any + AdvancedWindow>(rt: &mut Runtime) -> Result<(), String> {
    let size: [f32; 2] = rt.pop_vec4()?;
    let size: [u32; 2] = [size[0] as u32, size[1] as u32];
    unsafe { Current::<W>::new() }.set_size(size);
    Ok(())
}

pub fn window_position<W: Any + AdvancedWindow>(_rt: &mut Runtime) -> Result<Variable, String> {
    Ok(Variable::Vec4(if let Some(pos) = unsafe { Current::<W>::new() }.get_position() {
        [pos.x as f32, pos.y as f32, 0.0, 0.0]
    } else {
        [0.0 as f32; 4]
    }))
}

#[allow(non_snake_case)]
pub fn set_window__position<W: Any + AdvancedWindow>(rt: &mut Runtime) -> Result<(), String> {
    let pos: [f32; 2] = rt.pop_vec4()?;
    let pos: [i32; 2] = [pos[0] as i32, pos[1] as i32];
    unsafe { Current::<W>::new() }.set_position(pos);
    Ok(())
}

dyon_fn!{fn event_loop_ups() -> f64 {
    use piston::event_loop::{EventLoop, Events};

    unsafe { Current::<Events>::new() }.get_event_settings().ups as f64
}}

dyon_fn!{fn set_event_loop__ups(ups: f64) {
    use piston::event_loop::{EventLoop, Events};

    unsafe { Current::<Events>::new() }.set_ups(ups as u64);
}}

dyon_fn!{fn event_loop_upsreset() -> f64 {
    use piston::event_loop::{EventLoop, Events};

    unsafe { Current::<Events>::new() }.get_event_settings().ups_reset as f64
}}

dyon_fn!{fn set_event_loop__upsreset(ups_reset: f64) {
    use piston::event_loop::{EventLoop, Events};

    unsafe { Current::<Events>::new() }.set_ups_reset(ups_reset as u64);
}}

dyon_fn!{fn event_loop_maxfps() -> f64 {
    use piston::event_loop::{EventLoop, Events};

    unsafe { Current::<Events>::new() }.get_event_settings().max_fps as f64
}}

dyon_fn!{fn set_event_loop__maxfps(max_fps: f64) {
    use piston::event_loop::{EventLoop, Events};

    unsafe { Current::<Events>::new() }.set_max_fps(max_fps as u64);
}}

dyon_fn!{fn event_loop_swapbuffers() -> bool {
    use piston::event_loop::{EventLoop, Events};

    unsafe { Current::<Events>::new() }.get_event_settings().swap_buffers
}}

dyon_fn!{fn set_event_loop__swapbuffers(swap_buffers: bool) {
    use piston::event_loop::{EventLoop, Events};

    unsafe { Current::<Events>::new() }.set_swap_buffers(swap_buffers);
}}

pub fn swap_buffers<W: Any + Window>(_rt: &mut Runtime) -> Result<(), String> {
    unsafe { Current::<W>::new() }.swap_buffers();
    Ok(())
}

dyon_fn!{fn event_loop_benchmode() -> bool {
    use piston::event_loop::{EventLoop, Events};

    unsafe { Current::<Events>::new() }.get_event_settings().bench_mode
}}

dyon_fn!{fn set_event_loop__benchmode(bench_mode: bool) {
    use piston::event_loop::{EventLoop, Events};

    unsafe { Current::<Events>::new() }.set_bench_mode(bench_mode);
}}

dyon_fn!{fn event_loop_lazy() -> bool {
    use piston::event_loop::{EventLoop, Events};

    unsafe { Current::<Events>::new() }.get_event_settings().lazy
}}

dyon_fn!{fn set_event_loop__lazy(lazy: bool) {
    use piston::event_loop::{EventLoop, Events};

    unsafe { Current::<Events>::new() }.set_lazy(lazy);
}}

dyon_fn!{fn render() -> bool {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).render_args().is_some() }
}}

dyon_fn!{fn after_render() -> bool {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).after_render_args().is_some() }
}}

dyon_fn!{fn update() -> bool {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).update_args().is_some() }
}}

dyon_fn!{fn idle() -> bool {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).idle_args().is_some() }
}}

dyon_fn!{fn press() -> bool {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).press_args().is_some() }
}}

dyon_fn!{fn release() -> bool {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).release_args().is_some() }
}}

dyon_fn!{fn resize() -> bool {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).resize_args().is_some() }
}}

dyon_fn!{fn focus() -> bool {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).focus_args().is_some() }
}}

dyon_fn!{fn cursor() -> bool {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).cursor_args().is_some() }
}}

dyon_fn!{fn text() -> bool {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).text(|_| ()).is_some() }
}}

dyon_fn!{fn mouse_cursor() -> bool {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).mouse_cursor_args().is_some() }
}}

dyon_fn!{fn focus_arg() -> Option<bool> {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).focus_args() }
}}

dyon_fn!{fn cursor_arg() -> Option<bool> {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).cursor_args() }
}}

dyon_fn!{fn render_ext_dt() -> Option<f64> {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).render_args().map(|args| args.ext_dt) }
}}

dyon_fn!{fn update_dt() -> Option<f64> {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).update_args().map(|args| args.dt) }
}}

dyon_fn!{fn idle_dt() -> Option<f64> {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).idle_args().map(|args| args.dt) }
}}

dyon_fn!{fn mouse_cursor_pos() -> Option<Vec4> {
    unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).mouse_cursor_args().map(|pos| pos.into()) }
}}

pub fn press_keyboard_key(_rt: &mut Runtime) -> Result<Variable, String> {
    use dyon::embed::PushVariable;

    let e = unsafe { &*Current::<Option<Event>>::new() };
    if let &Some(ref e) = e {
        Ok(if let Some(Button::Keyboard(key)) = e.press_args() {
            Some(key as u64 as f64)
        } else {
            Option::<f64>::None
        }.push_var())
    } else {
        Err(NO_EVENT.into())
    }
}

pub fn release_keyboard_key(_rt: &mut Runtime) -> Result<Variable, String> {
    use dyon::embed::PushVariable;

    let e = unsafe { &*Current::<Option<Event>>::new() };
    if let &Some(ref e) = e {
        Ok(if let Some(Button::Keyboard(key)) = e.release_args() {
            Some(key as u64 as f64)
        } else {
            Option::<f64>::None
        }.push_var())
    } else {
        Err(NO_EVENT.into())
    }
}

pub fn press_mouse_button(_rt: &mut Runtime) -> Result<Variable, String> {
    use dyon::embed::PushVariable;

    let e = unsafe { &*Current::<Option<Event>>::new() };
    if let &Some(ref e) = e {
        Ok(if let Some(Button::Mouse(button)) = e.press_args() {
            Some(button as u64 as f64)
        } else {
            Option::<f64>::None
        }.push_var())
    } else {
        Err(NO_EVENT.into())
    }
}

pub fn release_mouse_button(_rt: &mut Runtime) -> Result<Variable, String> {
    use dyon::embed::PushVariable;

    let e = unsafe { &*Current::<Option<Event>>::new() };
    if let &Some(ref e) = e {
        Ok(if let Some(Button::Mouse(button)) = e.release_args() {
            Some(button as u64 as f64)
        } else {
            Option::<f64>::None
        }.push_var())
    } else {
        Err(NO_EVENT.into())
    }
}

pub fn text_arg(_rt: &mut Runtime) -> Result<Variable, String> {
    use dyon::embed::PushVariable;

    let e = unsafe { &*Current::<Option<Event>>::new() };
    if let &Some(ref e) = e {
        Ok(if let Some(text) = e.text_args() {
            Some(text)
        } else {
            Option::<String>::None
        }.push_var())
    } else {
        Err(NO_EVENT.into())
    }
}

pub fn window_title<W: Any + AdvancedWindow>(_rt: &mut Runtime) -> Result<Variable, String> {
    let window = unsafe { &mut *Current::<W>::new() };
    Ok(Variable::Str(Arc::new(window.get_title())))
}

#[allow(non_snake_case)]
pub fn set_window__title<W: Any + AdvancedWindow>(rt: &mut Runtime) -> Result<(), String> {
    let window = unsafe { &mut *Current::<W>::new() };
    let title: Arc<String> = rt.pop()?;
    window.set_title((*title).clone());
    Ok(())
}

#[allow(non_snake_case)]
pub fn width__font_size_string<C: Any + CharacterCache>(rt: &mut Runtime) -> Result<Variable, String> {
    let glyphs = unsafe { &mut *Current::<Vec<C>>::new() };
    let s: Arc<String> = rt.pop()?;
    let size: u32 = rt.pop()?;
    let font: usize = rt.pop()?;
    Ok(Variable::f64(glyphs.get_mut(font).ok_or_else(|| "Font index outside range".to_owned())?
        .width(size, &s).map_err(|_| "Could not get glyph".to_owned())?))
}

/// Wraps font names as a current object.
pub struct FontNames(pub Vec<Arc<String>>);

/// Wraps image names as a current object.
pub struct ImageNames(pub Vec<Arc<String>>);

dyon_fn!{fn font_names() -> Vec<Arc<String>> {
    let font_names = unsafe { &*Current::<FontNames>::new() };
    font_names.0.clone()
}}

/// Helper method for loading fonts.
pub fn load_font<F, T>(rt: &mut Runtime) -> Result<Variable, String>
    where F: 'static + Clone, T: 'static +
          CreateTexture<F> + UpdateTexture<F> +
          graphics::ImageSize
{
    use dyon::embed::PushVariable;
    use texture::{Filter, TextureSettings};
    use graphics::glyph_cache::rusttype::GlyphCache;

    let glyphs = unsafe { &mut *Current::<Vec<GlyphCache<'static, F, T>>>::new() };
    let font_names = unsafe { &mut *Current::<FontNames>::new() };
    let factory = unsafe { &*Current::<F>::new() };
    let file: Arc<String> = rt.pop()?;
    let texture_settings = TextureSettings::new().filter(Filter::Nearest);
    Ok(match GlyphCache::<'static, F, T>::new(&**file, factory.clone(), texture_settings) {
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

dyon_fn!{fn image_names() -> Vec<Arc<String>> {
    let image_names = unsafe { &*Current::<ImageNames>::new() };
    image_names.0.clone()
}}

dyon_fn!{fn load_image(file: Arc<String>) -> Result<usize, Arc<String>> {
    use image::{open, RgbaImage};

    let images = unsafe { &mut *Current::<Vec<RgbaImage>>::new() };
    let image_names = unsafe { &mut *Current::<ImageNames>::new() };
    match open(&**file) {
        Ok(x) => {
            let id = images.len();
            images.push(x.to_rgba8());
            image_names.0.push(file.clone());
            Ok(id)
        }
        Err(err) => {
            Err(Arc::new(format!("{}", err)))
        }
    }
}}

dyon_fn!{fn create_image__name_size(name: Arc<String>, size: Vec4) -> usize {
    use image::RgbaImage;

    let image_names = unsafe { &mut *Current::<ImageNames>::new() };
    let images = unsafe { &mut *Current::<Vec<RgbaImage>>::new() };
    let id = images.len();
    let size: [f64; 2] = size.into();
    image_names.0.push(name);
    images.push(RgbaImage::new(size[0] as u32, size[1] as u32));
    id
}}

dyon_fn!{fn save__image_file(id: usize, file: Arc<String>) -> Result<Arc<String>, Arc<String>> {
    use image::RgbaImage;

    let images = unsafe { &mut *Current::<Vec<RgbaImage>>::new() };
    match images[id].save(&**file) {
        Ok(_) => Ok(file),
        Err(err) => Err(Arc::new(format!("{}", err))),
    }
}}

dyon_fn!{fn image_size(id: usize) -> Vec4 {
    use image::RgbaImage;

    let images = unsafe { &*Current::<Vec<RgbaImage>>::new() };
    let (w, h) = images[id].dimensions();
    [w as f64, h as f64].into()
}}

#[allow(non_snake_case)]
pub fn pxl__image_pos_color(rt: &mut Runtime) -> Result<(), String> {
    use image::{Rgba, RgbaImage};

    let images = unsafe { &mut *Current::<Vec<RgbaImage>>::new() };
    let color: [f32; 4] = rt.pop_vec4()?;
    let pos: [f64; 2] = rt.pop_vec4()?;
    let id: usize = rt.pop()?;
    let image = if let Some(x) = images.get_mut(id) {
        x
    } else {
        return Err("Image id is out of bounds".into());
    };
    let x = pos[0] as u32;
    let y = pos[1] as u32;
    let (w, h) = image.dimensions();
    if x >= w || y >= h {
        return Err("Pixel is out of image bounds".into());
    }
    image.put_pixel(x, y, Rgba([
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
        (color[3] * 255.0) as u8
    ]));
    Ok(())
}

#[allow(non_snake_case)]
pub fn pxl__image_pos(rt: &mut Runtime) -> Result<Variable, String> {
    use image::RgbaImage;

    let images = unsafe { &*Current::<Vec<RgbaImage>>::new() };
    let pos: [f64; 2] = rt.pop_vec4()?;
    let id: usize = rt.pop()?;
    let image = if let Some(x) = images.get(id) {
        x
    } else {
        return Err("Image id is out of bounds".into());
    };
    let x = pos[0] as u32;
    let y = pos[1] as u32;
    let (w, h) = image.dimensions();
    if x >= w || y >= h {
        return Err("Pixel is out of image bounds".into());
    }
    let color = image.get_pixel(x, y).0;
    Ok(Variable::Vec4([
        color[0] as f32 / 255.0,
        color[1] as f32 / 255.0,
        color[2] as f32 / 255.0,
        color[3] as f32 / 255.0
    ]))
}

/// Helper method for drawing 2D in Dyon environment.
pub fn draw_2d<C: CharacterCache<Texture = G::Texture>, G: Graphics>(
    rt: &mut Runtime,
    glyphs: &mut Vec<C>,
    textures: &mut Vec<G::Texture>,
    mut c: Context,
    g: &mut G
) -> Result<(), String> {
    use self::graphics::*;
    use self::graphics::types::Matrix2d;
    use self::graphics::draw_state::{Blend, Stencil};

    let draw_list = rt.stack.pop().expect(TINVOTS);
    let arr = rt.get(&draw_list);
    let mut transform = c.transform;
    if let &Variable::Array(ref arr) = arr {
        for it in &**arr {
            let it = rt.get(it);
            if let &Variable::Array(ref it) = it {
                let ty: Arc<String> = rt.var(&it[0])?;
                match &**ty {
                    "clear" => {
                        let color: [f32; 4] = rt.var_vec4(&it[1])?;
                        clear(color, g);
                    }
                    "clear__stencil" => {
                        let v: f64 = rt.var(&it[1])?;
                        g.clear_stencil(v as u8);
                    }
                    "clear__colorbuf" => {
                        let color: [f32; 4] = rt.var_vec4(&it[1])?;
                        g.clear_color(color);
                    }
                    "transform__rx_ry" => {
                        // Changes transform matrix.
                        let rx: [f32; 4] = rt.var_vec4(&it[1])?;
                        let ry: [f32; 4] = rt.var_vec4(&it[2])?;
                        let t: Matrix2d = [
                            [rx[0] as f64, rx[1] as f64, rx[2] as f64],
                            [ry[0] as f64, ry[1] as f64, ry[2] as f64]
                        ];
                        transform = math::multiply(c.transform, t);
                    }
                    "rel_transform__rx_ry" => {
                        // Changes transform matrix.
                        let rx: [f32; 4] = rt.var_vec4(&it[1])?;
                        let ry: [f32; 4] = rt.var_vec4(&it[2])?;
                        let t: Matrix2d = [
                            [rx[0] as f64, rx[1] as f64, rx[2] as f64],
                            [ry[0] as f64, ry[1] as f64, ry[2] as f64]
                        ];
                        transform = math::multiply(transform, t);
                    }
                    "line__color_radius_from_to" => {
                        let color: [f32; 4] = rt.var_vec4(&it[1])?;
                        let radius: f64 = rt.var(&it[2])?;
                        let from: [f64; 2] = rt.var_vec4(&it[3])?;
                        let to: [f64; 2] = rt.var_vec4(&it[4])?;
                        Line::new(color, radius)
                            .draw([from[0], from[1], to[0], to[1]],
                                  &c.draw_state, transform, g);
                    }
                    "rectangle__color_corner_size" => {
                        let color: [f32; 4] = rt.var_vec4(&it[1])?;
                        let corner: [f64; 2] = rt.var_vec4(&it[2])?;
                        let size: [f64; 2] = rt.var_vec4(&it[3])?;
                        Rectangle::new(color)
                            .draw([corner[0], corner[1], size[0], size[1]],
                                  &c.draw_state, transform, g);
                    }
                    "rectangle__border_color_corner_size" => {
                        let radius: f64 = rt.var(&it[1])?;
                        let color: [f32; 4] = rt.var_vec4(&it[2])?;
                        let corner: [f64; 2] = rt.var_vec4(&it[3])?;
                        let size: [f64; 2] = rt.var_vec4(&it[4])?;
                        Rectangle::new_border(color, radius)
                            .draw([corner[0], corner[1], size[0], size[1]],
                                  &c.draw_state, transform, g);
                    }
                    "ellipse__color_corner_size_resolution" => {
                        let color: [f32; 4] = rt.var_vec4(&it[1])?;
                        let corner: [f64; 2] = rt.var_vec4(&it[2])?;
                        let size: [f64; 2] = rt.var_vec4(&it[3])?;
                        let resolution: u32 = rt.var(&it[4])?;
                        Ellipse::new(color)
                        .resolution(resolution as u32)
                        .draw([corner[0], corner[1], size[0], size[1]], &c.draw_state, transform, g);
                    }
                    "ellipse__border_color_corner_size_resolution" => {
                        let border: f64 = rt.var(&it[1])?;
                        let color: [f32; 4] = rt.var_vec4(&it[2])?;
                        let corner: [f64; 2] = rt.var_vec4(&it[3])?;
                        let size: [f64; 2] = rt.var_vec4(&it[4])?;
                        let resolution: u32 = rt.var(&it[5])?;
                        Ellipse::new_border(color, border)
                        .resolution(resolution as u32)
                        .draw([corner[0], corner[1], size[0], size[1]], &c.draw_state, transform, g);
                    }
                    "text__font_color_size_pos_string" => {
                        let font: usize = rt.var(&it[1])?;
                        let color: [f32; 4] = rt.var_vec4(&it[2])?;
                        let size: u32 = rt.var(&it[3])?;
                        let pos: [f64; 2] = rt.var_vec4(&it[4])?;
                        let text: Arc<String> = rt.var(&it[5])?;
                        text::Text::new_color(color, size).draw(
                            &text,
                            glyphs.get_mut(font)
                                .ok_or_else(|| "Font index outside range".to_owned())?,
                            &c.draw_state,
                            transform.trans(pos[0], pos[1]), g
                        ).map_err(|_| "Could not get glyph".to_owned())?;
                    }
                    "image__texture_pos_color" => {
                        let id: usize = rt.var(&it[1])?;
                        let pos: [f64; 2] = rt.var_vec4(&it[2])?;
                        let color: [f32; 4] = rt.var_vec4(&it[3])?;
                        Image::new_color(color).draw(
                            &textures[id],
                            &c.draw_state,
                            transform.trans(pos[0], pos[1]), g
                        );
                    }
                    "image__texture_pos_color_srccorner_srcsize" => {
                        let id: usize = rt.var(&it[1])?;
                        let pos: [f64; 2] = rt.var_vec4(&it[2])?;
                        let color: [f32; 4] = rt.var_vec4(&it[3])?;
                        let srccorner: [f64; 2] = rt.var_vec4(&it[4])?;
                        let srcsize: [f64; 2] = rt.var_vec4(&it[5])?;
                        Image::new_color(color)
                        .src_rect([srccorner[0], srccorner[1], srcsize[0], srcsize[1]])
                        .draw(
                            &textures[id],
                            &c.draw_state,
                            transform.trans(pos[0], pos[1]), g
                        );
                    }
                    "draw_state_alpha" => {
                        c.draw_state = DrawState::new_alpha();
                    }
                    "draw_state_clip" => {
                        c.draw_state = DrawState::new_clip();
                    }
                    "draw_state_increment" => {
                        c.draw_state = DrawState::new_increment();
                    }
                    "draw_state_inside" => {
                        c.draw_state = DrawState::new_inside();
                    }
                    "draw_state_outside" => {
                        c.draw_state = DrawState::new_outside();
                    }
                    "blend_alpha" => {
                        c.draw_state.blend = Some(Blend::Alpha);
                    }
                    "blend_add" => {
                        c.draw_state.blend = Some(Blend::Add);
                    }
                    "blend_lighter" => {
                        c.draw_state.blend = Some(Blend::Lighter);
                    }
                    "blend_multiply" => {
                        c.draw_state.blend = Some(Blend::Multiply);
                    }
                    "blend_invert" => {
                        c.draw_state.blend = Some(Blend::Invert);
                    }
                    "scissor__corner_size" => {
                        let corner: [f64; 2] = rt.var_vec4(&it[1])?;
                        let size: [f64; 2] = rt.var_vec4(&it[2])?;
                        c.draw_state.scissor = Some([
                            corner[0] as u32,
                            corner[1] as u32,
                            size[0] as u32,
                            size[1] as u32
                        ]);
                    }
                    "stencil__clip" => {
                        let v: f64 = rt.var(&it[1])?;
                        c.draw_state.stencil = Some(Stencil::Clip(v as u8));
                    }
                    "stencil__inside" => {
                        let v: f64 = rt.var(&it[1])?;
                        c.draw_state.stencil = Some(Stencil::Inside(v as u8));
                    }
                    "stencil__outside" => {
                        let v: f64 = rt.var(&it[1])?;
                        c.draw_state.stencil = Some(Stencil::Outside(v as u8));
                    }
                    "stencil_increment" => {
                        c.draw_state.stencil = Some(Stencil::Increment);
                    }
                    _ => {}
                }
            }
        }
    }
    return Ok(())
}
