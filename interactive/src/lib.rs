extern crate piston;
extern crate dyon;
extern crate current;
extern crate graphics;

use std::any::Any;
use std::sync::Arc;
use self::dyon::*;
use self::current::Current;
use self::piston::input::*;
use self::piston::window::*;
use self::graphics::{Context, Graphics};

pub const NO_EVENT: &'static str = "No event";

pub fn add_functions<W: Any + AdvancedWindow>(module: &mut Module) {
    module.add(Arc::new("render".into()), render, PreludeFunction {
        lts: vec![],
        tys: vec![],
        ret: Type::Bool
    });
    module.add(Arc::new("update".into()), update, PreludeFunction {
        lts: vec![],
        tys: vec![],
        ret: Type::Bool
    });
    module.add(Arc::new("press".into()), press, PreludeFunction {
        lts: vec![],
        tys: vec![],
        ret: Type::Bool
    });
    module.add(Arc::new("release".into()), release, PreludeFunction {
        lts: vec![],
        tys: vec![],
        ret: Type::Bool
    });
    module.add(Arc::new("focus".into()), focus, PreludeFunction {
        lts: vec![],
        tys: vec![],
        ret: Type::Bool,
    });
    module.add(Arc::new("focus_arg".into()), focus_arg, PreludeFunction {
        lts: vec![],
        tys: vec![],
        ret: Type::Option(Box::new(Bool)),
    });
    module.add(Arc::new("set_title".into()),
        set_title::<W>, PreludeFunction {
            lts: vec![Lt::Default],
            tys: vec![Type::Text],
            ret: Type::Void
        });
    module.add(Arc::new("update_dt".into()),
        update_dt, PreludeFunction {
            lts: vec![],
            tys: vec![],
            ret: Type::Option(Box::new(F64))
        });
    module.add(Arc::new("press_keyboard_key".into()),
        press_keyboard_key, PreludeFunction {
            lts: vec![],
            tys: vec![],
            ret: Type::Option(Box::new(Type::F64))
        });
    module.add(Arc::new("release_keyboard_key".into()),
        release_keyboard_key, PreludeFunction {
            lts: vec![],
            tys: vec![],
            ret: Type::Option(Box::new(Type::F64))
        });
}

pub fn render(rt: &mut Runtime) -> Result<(), String> {
    rt.push(unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).render_args().is_some() });
    Ok(())
}

pub fn update(rt: &mut Runtime) -> Result<(), String> {
    rt.push(unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).update_args().is_some() });
    Ok(())
}

pub fn press(rt: &mut Runtime) -> Result<(), String> {
    rt.push(unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).press_args().is_some() });
    Ok(())
}

pub fn release(rt: &mut Runtime) -> Result<(), String> {
    rt.push(unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).release_args().is_some() });
    Ok(())
}

pub fn focus(rt: &mut Runtime) -> Result<(), String> {
    rt.push(unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).focus_args().is_some() });
    Ok(())
}

pub fn focus_arg(rt: &mut Runtime) -> Result<(), String> {
    rt.push(unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).focus_args() });
    Ok(())
}

pub fn update_dt(rt: &mut Runtime) -> Result<(), String> {
    rt.push(unsafe { Current::<Option<Event>>::new()
        .as_ref().expect(NO_EVENT).update_args().map(|args| args.dt) });
    Ok(())
}

pub fn press_keyboard_key(rt: &mut Runtime) -> Result<(), String> {
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

pub fn set_title<W: Any + AdvancedWindow>(rt: &mut Runtime) -> Result<(), String> {
    use std::sync::Arc;

    let window = unsafe { &mut *Current::<W>::new() };
    let title: Arc<String> = try!(rt.pop());
    window.set_title((*title).clone());
    Ok(())
}

pub fn draw_2d<G: Graphics>(rt: &mut Runtime, c: Context, g: &mut G) -> Result<(), String> {
    use self::graphics::*;

    let draw_list = rt.stack.pop().expect("There is no value on the stack");
    let arr = rt.resolve(&draw_list);
    if let &Variable::Array(ref arr) = arr {
        for it in &**arr {
            let it = rt.resolve(it);
            if let &Variable::Array(ref it) = it {
                let ty: Arc<String> = try!(rt.var(&it[0]));
                match &**ty {
                    "clear" => {
                        let color: [f32; 4] = try!(rt.var_vec4(&it[1]));
                        clear(color, g);
                    }
                    "line_color_radius_from_to" => {
                        let color: [f32; 4] = try!(rt.var_vec4(&it[1]));
                        let radius: f64 = try!(rt.var(&it[2]));
                        let from: [f64; 2] = try!(rt.var_vec4(&it[3]));
                        let to: [f64; 2] = try!(rt.var_vec4(&it[4]));
                        line(color, radius, [from[0], from[1], to[0], to[1]], c.transform, g);
                    }
                    "rectangle_color_corner_size" => {
                        let color: [f32; 4] = try!(rt.var_vec4(&it[1]));
                        let corner: [f64; 2] = try!(rt.var_vec4(&it[2]));
                        let size: [f64; 2] = try!(rt.var_vec4(&it[3]));
                        rectangle(color, [corner[0], corner[1], size[0], size[1]], c.transform, g);
                    }
                    "ellipse_color_corner_size_resolution" => {
                        let color: [f32; 4] = try!(rt.var_vec4(&it[1]));
                        let corner: [f64; 2] = try!(rt.var_vec4(&it[2]));
                        let size: [f64; 2] = try!(rt.var_vec4(&it[3]));
                        let resolution: u32 = try!(rt.var(&it[4]));
                        Ellipse::new(color)
                        .resolution(resolution as u32)
                        .draw([corner[0], corner[1], size[0], size[1]], &c.draw_state, c.transform, g);
                    }
                    _ => {}
                }
            }
        }
    }
    return Ok(())
}
