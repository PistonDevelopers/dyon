#![allow(non_snake_case)]

use *;

mod io;
mod meta;
mod data;
mod lifetimechk;
mod functions;

#[cfg(not(feature = "http"))]
const HTTP_SUPPORT_DISABLED: &'static str = "Http support is disabled";

#[cfg(not(feature = "file"))]
const FILE_SUPPORT_DISABLED: &'static str = "File support is disabled";

dyon_fn!{fn x(v: Vec4) -> f64 {v.0[0] as f64}}
dyon_fn!{fn y(v: Vec4) -> f64 {v.0[1] as f64}}
dyon_fn!{fn z(v: Vec4) -> f64 {v.0[2] as f64}}
dyon_fn!{fn w(v: Vec4) -> f64 {v.0[3] as f64}}

pub(crate) fn s(rt: &mut Runtime) -> Result<(), String> {
    let ind: f64 = rt.pop().expect(TINVOTS);
    let ind = ind as usize;
    if ind >= 4 {return Err(format!("Index out of bounds `{}`", ind))};
    let v: [f32; 4] = rt.pop_vec4().expect(TINVOTS);
    rt.push(v[ind] as f64);
    Ok(())
}

dyon_fn!{fn det(m: Mat4) -> f64 {vecmath::mat4_det(m.0) as f64}}
dyon_fn!{fn inv(m: Mat4) -> Mat4 {Mat4(vecmath::mat4_inv(m.0))}}
dyon_fn!{fn mov(v: Vec4) -> Mat4 {Mat4([
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [v.0[0], v.0[1], v.0[2], 1.0],
])}}
dyon_fn!{fn rot__axis_angle(axis: Vec4, ang: f64) -> Mat4 {
    let axis = [axis.0[0] as f64, axis.0[1] as f64, axis.0[2] as f64];
    let cos = ang.cos();
    let sin = ang.sin();
    let inv_cos = 1.0 - cos;
    Mat4([
        [
            (cos + axis[0] * axis[0] * inv_cos) as f32,
            (axis[0] * axis[1] * inv_cos - axis[2] * sin) as f32,
            (axis[0] * axis[2] * inv_cos + axis[1] * sin) as f32,
            0.0
        ],
        [
            (axis[1] * axis[0] * inv_cos + axis[2] * sin) as f32,
            (cos + axis[1] * axis[1] * inv_cos) as f32,
            (axis[1] * axis[2] * inv_cos - axis[0] * sin) as f32,
            0.0
        ],
        [
            (axis[2] * axis[0] * inv_cos - axis[1] * sin) as f32,
            (axis[2] * axis[1] * inv_cos + axis[0] * sin) as f32,
            (cos + axis[2] * axis[2] * inv_cos) as f32,
            0.0
        ],
        [0.0,0.0,0.0,1.0]
    ])
}}
dyon_fn!{fn ortho__pos_right_up_forward(pos: Vec4, right: Vec4, up: Vec4, forward: Vec4) -> Mat4 {
    use vecmath::vec4_dot as dot;
    Mat4([
        [right.0[0], up.0[0], forward.0[0], 0.0],
        [right.0[1], up.0[1], forward.0[1], 0.0],
        [right.0[2], up.0[2], forward.0[2], 0.0],
        [-dot(right.0, pos.0), -dot(up.0, pos.0), -dot(forward.0, pos.0), 1.0],
    ])
}}
dyon_fn!{fn proj__fov_near_far_ar(fov: f64, near: f64, far: f64, ar: f64) -> Mat4 {
    let f = 1.0 / (fov * ::std::f64::consts::PI).tan();
    Mat4([
        [(f/ar) as f32, 0.0, 0.0, 0.0],
        [0.0, f as f32, 0.0, 0.0],
        [0.0, 0.0, ((far + near) / (near - far)) as f32, -1.0],
        [0.0, 0.0, ((2.0 * far * near) / (near - far)) as f32, 0.0],
    ])
}}
dyon_fn!{fn mvp__model_view_projection(model: Mat4, view: Mat4, proj: Mat4) -> Mat4 {
    use vecmath::col_mat4_mul as mul;
    Mat4(mul(mul(proj.0, view.0), model.0))
}}
dyon_fn!{fn scale(v: Vec4) -> Mat4 {Mat4([
    [v.0[0], 0.0, 0.0, 0.0],
    [0.0, v.0[1], 0.0, 0.0],
    [0.0, 0.0, v.0[2], 0.0],
    [0.0, 0.0, 0.0, 1.0],
])}}

dyon_fn!{fn rx(m: Mat4) -> Vec4 {Vec4([m.0[0][0], m.0[1][0], m.0[2][0], m.0[3][0]])}}
dyon_fn!{fn ry(m: Mat4) -> Vec4 {Vec4([m.0[0][1], m.0[1][1], m.0[2][1], m.0[3][1]])}}
dyon_fn!{fn rz(m: Mat4) -> Vec4 {Vec4([m.0[0][2], m.0[1][2], m.0[2][2], m.0[3][2]])}}
dyon_fn!{fn rw(m: Mat4) -> Vec4 {Vec4([m.0[0][3], m.0[1][3], m.0[2][3], m.0[3][3]])}}

pub(crate) fn rv(rt: &mut Runtime) -> Result<(), String> {
    let ind: f64 = rt.pop().expect(TINVOTS);
    let ind = ind as usize;
    if ind >= 4 {return Err(format!("Index out of bounds `{}`", ind))};
    let m: [[f32; 4]; 4] = rt.pop_mat4().expect(TINVOTS);
    rt.stack.push(Variable::Vec4([m[0][ind], m[1][ind], m[2][ind], m[3][ind]]));
    Ok(())
}

dyon_fn!{fn cx(m: Mat4) -> Vec4 {Vec4(m.0[0])}}
dyon_fn!{fn cy(m: Mat4) -> Vec4 {Vec4(m.0[1])}}
dyon_fn!{fn cz(m: Mat4) -> Vec4 {Vec4(m.0[2])}}
dyon_fn!{fn cw(m: Mat4) -> Vec4 {Vec4(m.0[3])}}

pub(crate) fn cv(rt: &mut Runtime) -> Result<(), String> {
    let ind: f64 = rt.pop().expect(TINVOTS);
    let ind = ind as usize;
    if ind >= 4 {return Err(format!("Index out of bounds `{}`", ind))};
    let m: [[f32; 4]; 4] = rt.pop_mat4().expect(TINVOTS);
    rt.stack.push(Variable::Vec4(m[ind]));
    Ok(())
}

pub(crate) fn clone(rt: &mut Runtime) -> Result<(), String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = rt.resolve(&v).deep_clone(&rt.stack);
    rt.stack.push(v);
    Ok(())
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn why(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = Variable::Array(Arc::new(match rt.resolve(&v) {
        &Variable::Bool(true, Some(ref sec)) => {
            let mut sec = (**sec).clone();
            sec.reverse();
            sec
        }
        &Variable::Bool(true, None) => {
            return Err(module.error(call.args[0].source_range(),
                &format!("{}\nThis does not make sense, perhaps an array is empty?",
                    rt.stack_trace()), rt))
        }
        &Variable::Bool(false, _) => {
            return Err(module.error(call.args[0].source_range(),
                &format!("{}\nMust be `true` to have meaning, try add or remove `!`",
                    rt.stack_trace()), rt))
        }
        x => return Err(module.error(call.args[0].source_range(),
            &rt.expected(x, "bool"), rt))
    }));
    Ok(Some(v))
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn _where(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = Variable::Array(Arc::new(match rt.resolve(&v) {
        &Variable::F64(val, Some(ref sec)) => {
            if val.is_nan() {
                return Err(module.error(call.args[0].source_range(),
                    &format!("{}\nExpected number, found `NaN`",
                        rt.stack_trace()), rt))
            } else {
                let mut sec = (**sec).clone();
                sec.reverse();
                sec
            }
        }
        &Variable::F64(_, None) => {
            return Err(module.error(call.args[0].source_range(),
                &format!("{}\nThis does not make sense, perhaps an array is empty?",
                    rt.stack_trace()), rt))
        }
        x => return Err(module.error(call.args[0].source_range(),
            &rt.expected(x, "f64"), rt))
    }));
    Ok(Some(v))
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn explain_why(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let why = rt.stack.pop().expect(TINVOTS);
    let val = rt.stack.pop().expect(TINVOTS);
    let (val, why) = match rt.resolve(&val) {
        &Variable::Bool(val, ref sec) => (val,
            match sec {
                &None => Box::new(vec![why.deep_clone(&rt.stack)]),
                &Some(ref sec) => {
                    let mut sec = sec.clone();
                    sec.push(why.deep_clone(&rt.stack));
                    sec
                }
            }
        ),
        x => return Err(module.error(call.args[0].source_range(),
            &rt.expected(x, "bool"), rt))
    };
    Ok(Some(Variable::Bool(val, Some(why))))
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn explain_where(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let wh = rt.stack.pop().expect(TINVOTS);
    let val = rt.stack.pop().expect(TINVOTS);
    let (val, wh) = match rt.resolve(&val) {
        &Variable::F64(val, ref sec) => (val,
            match sec {
                &None => Box::new(vec![wh.deep_clone(&rt.stack)]),
                &Some(ref sec) => {
                    let mut sec = sec.clone();
                    sec.push(wh.deep_clone(&rt.stack));
                    sec
                }
            }
        ),
        x => return Err(module.error(call.args[0].source_range(),
            &rt.expected(x, "bool"), rt))
    };
    Ok(Some(Variable::F64(val, Some(wh))))
}

pub(crate) fn println(rt: &mut Runtime) -> Result<(), String> {
    use write::{print_variable, EscapeString};

    let x = rt.stack.pop().expect(TINVOTS);
    print_variable(rt, &x, EscapeString::None);
    println!("");
    Ok(())
}

pub(crate) fn print(rt: &mut Runtime) -> Result<(), String> {
    use write::{print_variable, EscapeString};

    let x = rt.stack.pop().expect(TINVOTS);
    print_variable(rt, &x, EscapeString::None);
    Ok(())
}

dyon_fn!{fn sqrt(a: f64) -> f64 {a.sqrt()}}
dyon_fn!{fn sin(a: f64) -> f64 {a.sin()}}
dyon_fn!{fn asin(a: f64) -> f64 {a.asin()}}
dyon_fn!{fn cos(a: f64) -> f64 {a.cos()}}
dyon_fn!{fn acos(a: f64) -> f64 {a.acos()}}
dyon_fn!{fn tan(a: f64) -> f64 {a.tan()}}
dyon_fn!{fn atan(a: f64) -> f64 {a.atan()}}
dyon_fn!{fn atan2(y: f64, x: f64) -> f64 {y.atan2(x)}}
dyon_fn!{fn exp(a: f64) -> f64 {a.exp()}}
dyon_fn!{fn ln(a: f64) -> f64 {a.ln()}}
dyon_fn!{fn log2(a: f64) -> f64 {a.log2()}}
dyon_fn!{fn log10(a: f64) -> f64 {a.log10()}}
dyon_fn!{fn round(a: f64) -> f64 {a.round()}}
dyon_fn!{fn abs(a: f64) -> f64 {a.abs()}}
dyon_fn!{fn floor(a: f64) -> f64 {a.floor()}}
dyon_fn!{fn ceil(a: f64) -> f64 {a.ceil()}}
dyon_fn!{fn sleep(v: f64) {
    use std::thread::sleep;
    use std::time::Duration;

    let secs = v as u64;
    let nanos = (v.fract() * 1.0e9) as u32;
    sleep(Duration::new(secs, nanos));
}}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn head(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = Variable::Option(match rt.resolve(&v) {
        &Variable::Link(ref link) => link.head(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "link"), rt))
    });
    Ok(Some(v))
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn tip(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = Variable::Option(match rt.resolve(&v) {
        &Variable::Link(ref link) => link.tip(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "link"), rt))
    });
    Ok(Some(v))
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn tail(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = Variable::Link(Box::new(match rt.resolve(&v) {
        &Variable::Link(ref link) => link.tail(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "link"), rt))
    }));
    Ok(Some(v))
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn neck(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = Variable::Link(Box::new(match rt.resolve(&v) {
        &Variable::Link(ref link) => link.neck(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "link"), rt))
    }));
    Ok(Some(v))
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn is_empty(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(Variable::bool(match rt.resolve(&v) {
        &Variable::Link(ref link) => link.is_empty(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "link"), rt))
    })))
}

pub(crate) fn random(rt: &mut Runtime) -> Result<(), String> {
    use rand::Rng;

    let v: f64 = rt.rng.gen();
    rt.push(v);
    Ok(())
}

dyon_fn!{fn tau() -> f64 {6.283185307179586}}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn len(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = match rt.stack.pop() {
        Some(v) => v,
        None => panic!(TINVOTS)
    };

    let v = {
        let arr = match rt.resolve(&v) {
            &Variable::Array(ref arr) => arr,
            x => return Err(module.error(call.args[0].source_range(),
                            &rt.expected(x, "array"), rt))
        };
        Variable::f64(arr.len() as f64)
    };
    Ok(Some(v))
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn push_ref(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let item = rt.stack.pop().expect(TINVOTS);
    let v = rt.stack.pop().expect(TINVOTS);

    if let Variable::Ref(ind) = v {
        let ok = if let Variable::Array(ref mut arr) = rt.stack[ind] {
            Arc::make_mut(arr).push(item);
            true
        } else {
            false
        };
        if !ok {
            return Err(module.error(call.args[0].source_range(),
                &format!("{}\nExpected reference to array",
                    rt.stack_trace()), rt));
        }
    } else {
        return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected reference to array",
                rt.stack_trace()), rt));
    }
    Ok(None)
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn insert_ref(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let item = rt.stack.pop().expect(TINVOTS);
    let index = rt.stack.pop().expect(TINVOTS);
    let index = match *rt.resolve(&index) {
        Variable::F64(index, _) => index,
        _ => return Err(module.error(call.args[1].source_range(),
                        &format!("{}\nExpected number",
                            rt.stack_trace()), rt))
    };
    let v = rt.stack.pop().expect(TINVOTS);

    if let Variable::Ref(ind) = v {
        if let Variable::Array(ref arr) = rt.stack[ind] {
            let index = index as usize;
            if index > arr.len() {
                return Err(module.error(call.source_range,
                            &format!("{}\nIndex out of bounds",
                            rt.stack_trace()), rt))
            }
        }
        let ok = if let Variable::Array(ref mut arr) = rt.stack[ind] {
            Arc::make_mut(arr).insert(index as usize, item);
            true
        } else {
            false
        };
        if !ok {
            return Err(module.error(call.args[0].source_range(),
                &format!("{}\nExpected reference to array",
                    rt.stack_trace()), rt));
        }
    } else {
        return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected reference to array",
                rt.stack_trace()), rt));
    }
    Ok(None)
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn push(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let item = rt.stack.pop().expect(TINVOTS);
    let item = rt.resolve(&item).deep_clone(&rt.stack);
    let v = rt.stack.pop().expect(TINVOTS);

    if let Variable::Ref(ind) = v {
        let ok = if let Variable::Array(ref mut arr) = rt.stack[ind] {
            Arc::make_mut(arr).push(item);
            true
        } else {
            false
        };
        if !ok {
            return Err(module.error(call.args[0].source_range(),
                &format!("{}\nExpected reference to array",
                    rt.stack_trace()), rt));
        }
    } else {
        return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected reference to array",
                rt.stack_trace()), rt));
    }
    Ok(None)
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn insert(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>
) -> Result<Option<Variable>, String> {
    let item = rt.stack.pop().expect(TINVOTS);
    let item = rt.resolve(&item).deep_clone(&rt.stack);
    let index = rt.stack.pop().expect(TINVOTS);
    let index = match *rt.resolve(&index) {
        Variable::F64(index, _) => index,
        _ => return Err(module.error(call.args[1].source_range(),
                        &format!("{}\nExpected number",
                            rt.stack_trace()), rt))
    };
    let v = rt.stack.pop().expect(TINVOTS);

    if let Variable::Ref(ind) = v {
        if let Variable::Array(ref arr) = rt.stack[ind] {
            let index = index as usize;
            if index > arr.len() {
                return Err(module.error(call.source_range,
                            &format!("{}\nIndex out of bounds",
                            rt.stack_trace()), rt))
            }
        }
        let ok = if let Variable::Array(ref mut arr) = rt.stack[ind] {
            Arc::make_mut(arr).insert(index as usize, item);
            true
        } else {
            false
        };
        if !ok {
            return Err(module.error(call.args[0].source_range(),
                &format!("{}\nExpected reference to array",
                    rt.stack_trace()), rt));
        }
    } else {
        return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected reference to array",
                rt.stack_trace()), rt));
    }
    Ok(None)
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn pop(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let arr = rt.stack.pop().expect(TINVOTS);
    let mut v: Option<Variable> = None;
    if let Variable::Ref(ind) = arr {
        let ok = if let Variable::Array(ref mut arr) = rt.stack[ind] {
            v = Arc::make_mut(arr).pop();
            true
        } else {
            false
        };
        if !ok {
            return Err(module.error(call.args[0].source_range(),
                &format!("{}\nExpected reference to array",
                    rt.stack_trace()), rt));
        }
    } else {
        return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected reference to array",
                rt.stack_trace()), rt));
    }
    let v = match v {
        None => return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected non-empty array",
                rt.stack_trace()), rt)),
        Some(val) => val
    };
    Ok(Some(v))
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn remove(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let index = rt.stack.pop().expect(TINVOTS);
    let index = match *rt.resolve(&index) {
        Variable::F64(index, _) => index,
        _ => return Err(module.error(call.args[1].source_range(),
                        &format!("{}\nExpected number",
                            rt.stack_trace()), rt))
    };
    let arr = rt.stack.pop().expect(TINVOTS);
    if let Variable::Ref(ind) = arr {
        if let Variable::Array(ref arr) = rt.stack[ind] {
            let index = index as usize;
            if index >= arr.len() {
                return Err(module.error(call.source_range,
                            &format!("{}\nIndex out of bounds",
                            rt.stack_trace()), rt))
            }
        }
        if let Variable::Array(ref mut arr) = rt.stack[ind] {
            return Ok(Some(Arc::make_mut(arr).remove(index as usize)));
        } else {
            false
        };
        return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected reference to array",
                rt.stack_trace()), rt));
    } else {
        return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected reference to array",
                rt.stack_trace()), rt));
    }
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn reverse(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    if let Variable::Ref(ind) = v {
        let ok = if let Variable::Array(ref mut arr) = rt.stack[ind] {
            Arc::make_mut(arr).reverse();
            true
        } else {
            false
        };
        if !ok {
            return Err(module.error(call.args[0].source_range(),
                &format!("{}\nExpected reference to array",
                    rt.stack_trace()), rt));
        }
    } else {
        return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected reference to array",
                rt.stack_trace()), rt));
    }
    Ok(None)
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn clear(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    if let Variable::Ref(ind) = v {
        let ok = if let Variable::Array(ref mut arr) = rt.stack[ind] {
            Arc::make_mut(arr).clear();
            true
        } else {
            false
        };
        if !ok {
            return Err(module.error(call.args[0].source_range(),
                &format!("{}\nExpected reference to array",
                    rt.stack_trace()), rt));
        }
    } else {
        return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected reference to array",
                rt.stack_trace()), rt));
    }
    Ok(None)
}

// TODO: Can't be rewritten as external function because it reports error on arguments.
pub(crate) fn swap(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let j = rt.stack.pop().expect(TINVOTS);
    let i = rt.stack.pop().expect(TINVOTS);
    let j = match rt.resolve(&j) {
        &Variable::F64(val, _) => val,
        x => return Err(module.error(call.args[2].source_range(),
            &rt.expected(x, "number"), rt))
    };
    let i = match rt.resolve(&i) {
        &Variable::F64(val, _) => val,
        x => return Err(module.error(call.args[1].source_range(),
            &rt.expected(x, "number"), rt))
    };
    let v = rt.stack.pop().expect(TINVOTS);
    if let Variable::Ref(ind) = v {
        let ok = if let Variable::Array(ref mut arr) = rt.stack[ind] {
            Arc::make_mut(arr).swap(i as usize, j as usize);
            true
        } else {
            false
        };
        if !ok {
            return Err(module.error(call.args[0].source_range(),
                &format!("{}\nExpected reference to array",
                    rt.stack_trace()), rt));
        }
    } else {
        return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected reference to array",
                rt.stack_trace()), rt));
    }
    Ok(None)
}

pub(crate) fn read_line(rt: &mut Runtime) -> Result<(), String> {
    use std::io::{self, Write};
    use std::error::Error;

    let mut input = String::new();
    io::stdout().flush().unwrap();
    let error = match io::stdin().read_line(&mut input) {
        Ok(_) => None,
        Err(error) => Some(error)
    };
    rt.push(if let Some(error) = error {
        return Err(error.description().into())
    } else {
        Variable::Text(Arc::new(input))
    });
    Ok(())
}

pub(crate) fn read_number(rt: &mut Runtime) -> Result<(), String> {
    use std::io::{self, Write};
    use std::error::Error;

    let err: Arc<String> = rt.pop().expect(TINVOTS);
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();
    let rv = loop {
        input.clear();
        stdout.flush().unwrap();
        match stdin.read_line(&mut input) {
            Ok(_) => {}
            Err(error) => return Err(error.description().into()),
        };
        match input.trim().parse::<f64>() {
            Ok(v) => break v,
            Err(_) => println!("{}", err),
        }
    };
    rt.push(rv);
    Ok(())
}

dyon_fn!{fn parse_number(text: Arc<String>) -> Option<f64> {text.trim().parse::<f64>().ok()}}
dyon_fn!{fn trim(v: Arc<String>) -> Arc<String> {Arc::new(v.trim().into())}}
dyon_fn!{fn trim_left(v: Arc<String>) -> Arc<String> {Arc::new(v.trim_left().into())}}
dyon_fn!{fn trim_right(v: Arc<String>) -> Arc<String> {Arc::new(v.trim_right().into())}}

pub(crate) fn _str(rt: &mut Runtime) -> Result<(), String> {
    use write::{write_variable, EscapeString};

    let v = rt.stack.pop().expect(TINVOTS);
    let mut buf: Vec<u8> = vec![];
    write_variable(&mut buf, rt, rt.resolve(&v), EscapeString::None, 0).unwrap();
    rt.push(String::from_utf8(buf).unwrap());
    Ok(())
}

pub(crate) fn json_string(rt: &mut Runtime) -> Result<(), String> {
    use write::{write_variable, EscapeString};

    let v = rt.stack.pop().expect(TINVOTS);
    let mut buf: Vec<u8> = vec![];
    write_variable(&mut buf, rt, rt.resolve(&v), EscapeString::Json, 0).unwrap();
    rt.stack.push(Variable::Text(Arc::new(String::from_utf8(buf).unwrap())));
    Ok(())
}

dyon_fn!{fn str__color(v: Vec4) -> Arc<String> {
    let v = v.0;
    let mut buf: Vec<u8> = vec![];
    let clamp = |x| {
        if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x }
    };
    let r = (clamp(v[0]) * 255.0) as usize;
    let g = (clamp(v[1]) * 255.0) as usize;
    let b = (clamp(v[2]) * 255.0) as usize;
    let a = (clamp(v[3]) * 255.0) as usize;
    let map = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
               'a', 'b', 'c', 'd', 'e', 'f'];
    let (r1, r2) = (r >> 4, r & 0xf);
    let (g1, g2) = (g >> 4, g & 0xf);
    let (b1, b2) = (b >> 4, b & 0xf);
    let (a1, a2) = (a >> 4, a & 0xf);
    buf.push('#' as u8);
    buf.push(map[r1] as u8); buf.push(map[r2] as u8);
    buf.push(map[g1] as u8); buf.push(map[g2] as u8);
    buf.push(map[b1] as u8); buf.push(map[b2] as u8);
    if a != 255 {
        buf.push(map[a1] as u8); buf.push(map[a2] as u8);
    }
    Arc::new(String::from_utf8(buf).unwrap())
}}

dyon_fn!{fn srgb_to_linear__color(v: Vec4) -> Vec4 {
    let v = v.0;
    let to_linear = |f: f32| {
        if f <= 0.04045 {
            f / 12.92
        } else {
            ((f + 0.055) / 1.055).powf(2.4)
        }
    };
    Vec4([to_linear(v[0]), to_linear(v[1]), to_linear(v[2]), v[3]])
}}

dyon_fn!{fn linear_to_srgb__color(v: Vec4) -> Vec4 {
    let v = v.0;
    let to_srgb = |f: f32| {
        if f <= 0.0031308 {
            f * 12.92
        } else {
            1.055 * f.powf(1.0 / 2.4) - 0.055
        }
    };
    Vec4([to_srgb(v[0]), to_srgb(v[1]), to_srgb(v[2]), v[3]])
}}

pub(crate) fn _typeof(rt: &mut Runtime) -> Result<(), String> {
    use crate::runtime::*;

    let v = rt.stack.pop().expect(TINVOTS);
    let t = Variable::Text(match rt.resolve(&v) {
        &Variable::Text(_) => text_type.clone(),
        &Variable::F64(_, _) => f64_type.clone(),
        &Variable::Vec4(_) => vec4_type.clone(),
        &Variable::Mat4(_) => mat4_type.clone(),
        &Variable::Return => return_type.clone(),
        &Variable::Bool(_, _) => bool_type.clone(),
        &Variable::Object(_) => object_type.clone(),
        &Variable::Array(_) => array_type.clone(),
        &Variable::Link(_) => link_type.clone(),
        &Variable::Ref(_) => ref_type.clone(),
        &Variable::UnsafeRef(_) => unsafe_ref_type.clone(),
        &Variable::RustObject(_) => rust_object_type.clone(),
        &Variable::Option(_) => option_type.clone(),
        &Variable::Result(_) => result_type.clone(),
        &Variable::Thread(_) => thread_type.clone(),
        &Variable::Closure(_, _) => closure_type.clone(),
        &Variable::In(_) => in_type.clone(),
    });
    rt.stack.push(t);
    Ok(())
}

pub(crate) fn debug(rt: &mut Runtime) -> Result<(), String> {
    println!("Stack {:#?}", rt.stack);
    println!("Locals {:#?}", rt.local_stack);
    println!("Currents {:#?}", rt.current_stack);
    Ok(())
}

pub(crate) fn backtrace(rt: &mut Runtime) -> Result<(), String> {
    println!("{:#?}", rt.call_stack);
    Ok(())
}

// TODO: Can't be rewritten as an external function because it uses the current module.
pub(crate) fn load(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use load;

    let v = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&v) {
        &Variable::Text(ref text) => {
            let mut m = Module::new_intrinsics(module.intrinsics.clone());
            for f in &module.ext_prelude {
                m.add(f.name.clone(), f.f, f.p.clone());
            }
            if let Err(err) = load(text, &mut m) {
                Variable::Result(Err(Box::new(Error {
                    message: Variable::Text(Arc::new(
                        format!("{}\n{}\n{}", rt.stack_trace(), err,
                            module.error(call.args[0].source_range(),
                            "When attempting to load module:", rt)))),
                    trace: vec![]
                })))
            } else {
                Variable::Result(Ok(Box::new(
                    Variable::RustObject(Arc::new(Mutex::new(Arc::new(m)))))))
            }
        }
        x => return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "string"), rt))
    };
    Ok(Some(v))
}

// TODO: Can't be rewritten as an external function because it uses the current module.
pub(crate) fn load__source_imports(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use load;

    let modules = rt.stack.pop().expect(TINVOTS);
    let source = rt.stack.pop().expect(TINVOTS);
    let mut new_module = Module::new_intrinsics(module.intrinsics.clone());
    for f in &module.ext_prelude {
        new_module.add(f.name.clone(), f.f, f.p.clone());
    }
    match rt.resolve(&modules) {
        &Variable::Array(ref array) => {
            for it in &**array {
                match rt.resolve(it) {
                    &Variable::RustObject(ref obj) => {
                        match obj.lock().unwrap().downcast_ref::<Arc<Module>>() {
                            Some(m) => {
                                // Add external functions from imports.
                                for f in &m.ext_prelude {
                                    let has_external = new_module.ext_prelude.iter()
                                        .any(|a| a.name == f.name);
                                    if !has_external {
                                        new_module.add(f.name.clone(), f.f, f.p.clone());
                                    }
                                }
                                // Register loaded functions from imports.
                                for f in &m.functions {
                                    new_module.register(f.clone())
                                }
                            }
                            None => return Err(module.error(
                                call.args[1].source_range(),
                                &format!("{}\nExpected `Module`",
                                    rt.stack_trace()), rt))
                        }
                    }
                    x => return Err(module.error(
                        call.args[1].source_range(),
                        &rt.expected(x, "Module"), rt))
                }
            }
        }
        x => return Err(module.error(call.args[1].source_range(),
                &rt.expected(x, "[Module]"), rt))
    }
    let v = match rt.resolve(&source) {
        &Variable::Text(ref text) => {
            if let Err(err) = load(text, &mut new_module) {
                Variable::Result(Err(Box::new(Error {
                    message: Variable::Text(Arc::new(
                        format!("{}\n{}\n{}", rt.stack_trace(), err,
                            module.error(call.args[0].source_range(),
                            "When attempting to load module:", rt)))),
                    trace: vec![]
                })))
            } else {
                Variable::Result(Ok(Box::new(
                    Variable::RustObject(Arc::new(
                        Mutex::new(Arc::new(new_module)))))))
            }
        }
        x => return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "[Module]"), rt))
    };
    Ok(Some(v))
}

// TODO: Can't be rewritten as an external function because it uses the current module.
pub(crate) fn module__in_string_imports(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use load_str;

    let modules = rt.stack.pop().expect(TINVOTS);
    let source = rt.stack.pop().expect(TINVOTS);
    let source = match rt.resolve(&source) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[1].source_range(),
                &rt.expected(x, "str"), rt))
    };
    let name = rt.stack.pop().expect(TINVOTS);
    let name = match rt.resolve(&name) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "str"), rt))
    };
    let mut new_module = Module::new_intrinsics(module.intrinsics.clone());
    for f in &module.ext_prelude {
        new_module.add(f.name.clone(), f.f, f.p.clone());
    }
    match rt.resolve(&modules) {
        &Variable::Array(ref array) => {
            for it in &**array {
                match rt.resolve(it) {
                    &Variable::RustObject(ref obj) => {
                        match obj.lock().unwrap().downcast_ref::<Arc<Module>>() {
                            Some(m) => {
                                // Add external functions from imports.
                                for f in &m.ext_prelude {
                                    let has_external = new_module.ext_prelude.iter()
                                        .any(|a| a.name == f.name);
                                    if !has_external {
                                        new_module.add(f.name.clone(), f.f, f.p.clone());
                                    }
                                }
                                // Register loaded functions from imports.
                                for f in &m.functions {
                                    new_module.register(f.clone())
                                }
                            }
                            None => return Err(module.error(
                                call.args[2].source_range(),
                                &format!("{}\nExpected `Module`",
                                    rt.stack_trace()), rt))
                        }
                    }
                    x => return Err(module.error(
                        call.args[2].source_range(),
                        &rt.expected(x, "Module"), rt))
                }
            }
        }
        x => return Err(module.error(call.args[2].source_range(),
                &rt.expected(x, "[Module]"), rt))
    }
    let v = if let Err(err) = load_str(&name, source, &mut new_module) {
            Variable::Result(Err(Box::new(Error {
                message: Variable::Text(Arc::new(
                    format!("{}\n{}\n{}", rt.stack_trace(), err,
                        module.error(call.args[0].source_range(),
                        "When attempting to load module:", rt)))),
                trace: vec![]
            })))
        } else {
            Variable::Result(Ok(Box::new(
                Variable::RustObject(Arc::new(
                    Mutex::new(Arc::new(new_module)))))))
        };
    Ok(Some(v))
}

// TODO: Can't be rewritten as an external function because it uses the current module.
pub(crate) fn _call(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    // Use the source from calling function.
    let source = module.functions[rt.call_stack.last().unwrap().index].source.clone();
    let args = rt.stack.pop().expect(TINVOTS);
    let fn_name = rt.stack.pop().expect(TINVOTS);
    let call_module = rt.stack.pop().expect(TINVOTS);
    let fn_name = match rt.resolve(&fn_name) {
        &Variable::Text(ref text) => text.clone(),
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "text"), rt))
    };
    let args = match rt.resolve(&args) {
        &Variable::Array(ref arr) => arr.clone(),
        x => return Err(module.error(call.args[2].source_range(),
                        &rt.expected(x, "array"), rt))
    };
    let obj = match rt.resolve(&call_module) {
        &Variable::RustObject(ref obj) => obj.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "Module"), rt))
    };

    match obj.lock().unwrap()
        .downcast_ref::<Arc<Module>>() {
        Some(m) => {
            use std::cell::Cell;

            let f_index = m.find_function(&fn_name, 0);
            match f_index {
                FnIndex::Loaded(f_index) => {
                    let f = &m.functions[f_index as usize];
                    if f.args.len() != args.len() {
                        return Err(module.error(
                            call.args[2].source_range(),
                            &format!(
                                "{}\nExpected `{}` arguments, found `{}`",
                                rt.stack_trace(),
                                f.args.len(), args.len()), rt))
                    }
                    try!(lifetimechk::check(f, &args).map_err(|err|
                        module.error(call.args[2].source_range(),
                        &format!("{}\n{}", err, rt.stack_trace()), rt)));
                }
                FnIndex::Intrinsic(_) | FnIndex::None |
                FnIndex::ExternalVoid(_) | FnIndex::ExternalReturn(_) =>
                    return Err(module.error(
                            call.args[1].source_range(),
                            &format!(
                                "{}\nCould not find function `{}`",
                                rt.stack_trace(),
                                fn_name), rt))
            }
            let call = ast::Call {
                alias: None,
                name: fn_name.clone(),
                f_index: Cell::new(f_index),
                args: args.iter().map(|arg|
                    ast::Expression::Variable(
                        call.source_range, arg.clone())).collect(),
                custom_source: Some(source),
                source_range: call.source_range,
            };

            try!(rt.call(&call, &m));
        }
        None => return Err(module.error(call.args[0].source_range(),
                    &format!("{}\nExpected `Module`",
                        rt.stack_trace()), rt))
    }

    Ok(None)
}

// TODO: Can't be rewritten as an external function because it uses the current module.
pub(crate) fn call_ret(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    // Use the source from calling function.
    let source = module.functions[rt.call_stack.last().unwrap().index].source.clone();
    let args = rt.stack.pop().expect(TINVOTS);
    let fn_name = rt.stack.pop().expect(TINVOTS);
    let call_module = rt.stack.pop().expect(TINVOTS);
    let fn_name = match rt.resolve(&fn_name) {
        &Variable::Text(ref text) => text.clone(),
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "text"), rt))
    };
    let args = match rt.resolve(&args) {
        &Variable::Array(ref arr) => arr.clone(),
        x => return Err(module.error(call.args[2].source_range(),
                        &rt.expected(x, "array"), rt))
    };
    let obj = match rt.resolve(&call_module) {
        &Variable::RustObject(ref obj) => obj.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "Module"), rt))
    };

    let v = match obj.lock().unwrap()
        .downcast_ref::<Arc<Module>>() {
        Some(m) => {
            use std::cell::Cell;

            let f_index = m.find_function(&fn_name, 0);
            match f_index {
                FnIndex::Loaded(f_index) => {
                    let f = &m.functions[f_index as usize];
                    if f.args.len() != args.len() {
                        return Err(module.error(
                            call.args[2].source_range(),
                            &format!(
                                "{}\nExpected `{}` arguments, found `{}`",
                                rt.stack_trace(),
                                f.args.len(), args.len()), rt))
                    }
                    try!(lifetimechk::check(f, &args).map_err(|err|
                        module.error(call.args[2].source_range(),
                        &format!("{}\n{}", err, rt.stack_trace()), rt)));
                }
                FnIndex::Intrinsic(_) | FnIndex::None |
                FnIndex::ExternalVoid(_) | FnIndex::ExternalReturn(_) =>
                    return Err(module.error(
                        call.args[1].source_range(),
                        &format!(
                            "{}\nCould not find function `{}`",
                            rt.stack_trace(),
                            fn_name), rt))
            }
            let call = ast::Call {
                alias: None,
                name: fn_name.clone(),
                f_index: Cell::new(f_index),
                args: args.iter().map(|arg|
                    ast::Expression::Variable(
                        call.source_range, arg.clone())).collect(),
                custom_source: Some(source),
                source_range: call.source_range,
            };

            try!(rt.call(&call, &m)).0
        }
        None => return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected `Module`", rt.stack_trace()), rt))
    };

    Ok(v)
}

// TODO: Can't be rewritten as an external function because it uses the current module.
pub(crate) fn functions(
    _rt: &mut Runtime,
    _call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    // List available functions in scope.
    let v = Variable::Array(Arc::new(functions::list_functions(module)));
    Ok(Some(v))
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn functions__module(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    // List available functions in scope.
    let m = rt.stack.pop().expect(TINVOTS);
    let m = match rt.resolve(&m) {
        &Variable::RustObject(ref obj) => obj.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "Module"), rt))
    };

    let functions = match m.lock().unwrap()
        .downcast_ref::<Arc<Module>>() {
        Some(m) => functions::list_functions(m),
        None => return Err(module.error(call.args[0].source_range(),
            &format!("{}\nExpected `Module`", rt.stack_trace()), rt))
    };

    let v = Variable::Array(Arc::new(functions));
    Ok(Some(v))
}

dyon_fn!{fn none() -> Option<Variable> {None}}

pub(crate) fn some(rt: &mut Runtime) -> Result<(), String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = Variable::Option(Some(Box::new(
        rt.resolve(&v).deep_clone(&rt.stack)
    )));
    rt.stack.push(v);
    Ok(())
}

pub(crate) fn ok(rt: &mut Runtime) -> Result<(), String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = Variable::Result(Ok(Box::new(
        rt.resolve(&v).deep_clone(&rt.stack)
    )));
    rt.stack.push(v);
    Ok(())
}

pub(crate) fn err(rt: &mut Runtime) -> Result<(), String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = Variable::Result(Err(Box::new(
        Error {
            message: rt.resolve(&v).deep_clone(&rt.stack),
            trace: vec![]
        })));
    rt.stack.push(v);
    Ok(())
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn is_err(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Result(Err(_)) => Variable::bool(true),
        &Variable::Result(Ok(_)) => Variable::bool(false),
        x => {
            return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "result"), rt));
        }
    }))
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn is_ok(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Result(Err(_)) => Variable::bool(false),
        &Variable::Result(Ok(_)) => Variable::bool(true),
        x => {
            return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "result"), rt));
        }
    }))
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn min(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&v) {
        &Variable::Array(ref arr) => {
            let mut min: f64 = ::std::f64::NAN;
            for v in &**arr {
                if let &Variable::F64(val, _) = rt.resolve(v) {
                    if val < min || min.is_nan() { min = val }
                }
            }
            min
        }
        x => {
            return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "array"), rt));
        }
    };
    Ok(Some(Variable::f64(v)))
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn max(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&v) {
        &Variable::Array(ref arr) => {
            let mut max: f64 = ::std::f64::NAN;
            for v in &**arr {
                if let &Variable::F64(val, _) = rt.resolve(v) {
                    if val > max || max.is_nan() { max = val }
                }
            }
            max
        }
        x => {
            return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "array"), rt));
        }
    };
    Ok(Some(Variable::f64(v)))
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn unwrap(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use write::{write_variable, EscapeString};

    // Return value does not depend on lifetime of argument since
    // `ok(x)` and `some(x)` perform a deep clone.
    let v = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&v) {
        &Variable::Option(Some(ref v)) => (**v).clone(),
        &Variable::Option(None) => {
            return Err(module.error(call.args[0].source_range(),
                &format!("{}\nExpected `some(_)`",
                    rt.stack_trace()), rt));
        }
        &Variable::Result(Ok(ref ok)) => (**ok).clone(),
        &Variable::Result(Err(ref err)) => {
            use std::str::from_utf8;

            // Print out error message.
            let mut w: Vec<u8> = vec![];
            w.extend_from_slice(rt.stack_trace().as_bytes());
            w.extend_from_slice("\n".as_bytes());
            write_variable(&mut w, rt, &err.message,
                           EscapeString::None, 0).unwrap();
            for t in &err.trace {
                w.extend_from_slice("\n".as_bytes());
                w.extend_from_slice(t.as_bytes());
            }
            return Err(module.error(call.args[0].source_range(),
                                    from_utf8(&w).unwrap(), rt));
        }
        x => {
            return Err(module.error(call.args[0].source_range(),
                                    &rt.expected(x, "some(_) or ok(_)"), rt));
        }
    };
    Ok(Some(v))
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn unwrap_or(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    // Return value does not depend on lifetime of argument since
    // `ok(x)` and `some(x)` perform a deep clone.
    let def = rt.stack.pop().expect(TINVOTS);
    let v = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&v) {
        &Variable::Option(Some(ref v)) => (**v).clone(),
        &Variable::Result(Ok(ref ok)) => (**ok).clone(),
        &Variable::Option(None) |
        &Variable::Result(Err(_)) => rt.resolve(&def).clone(),
        x => {
            return Err(module.error(call.args[0].source_range(),
                                    &rt.expected(x, "some(_) or ok(_)"), rt));
        }
    };
    Ok(Some(v))
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn unwrap_err(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Result(Err(ref err)) => err.message.clone(),
        x => {
            return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "err(_)"), rt));
        }
    }))
}

dyon_fn!{fn dir__angle(val: f64) -> Vec4 {Vec4([val.cos() as f32, val.sin() as f32, 0.0, 0.0])}}

dyon_fn!{fn load__meta_file(meta: Arc<String>, file: Arc<String>) -> Variable {
    let res = meta::load_meta_file(&**meta, &**file);
    Variable::Result(match res {
        Ok(res) => Ok(Box::new(Variable::Array(Arc::new(res)))),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(err)),
            trace: vec![]
        }))
    })
}}

dyon_fn!{fn load__meta_url(meta: Arc<String>, url: Arc<String>) -> Variable {
    let res = meta::load_meta_url(&**meta, &**url);
    Variable::Result(match res {
        Ok(res) => Ok(Box::new(Variable::Array(Arc::new(res)))),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(err)),
            trace: vec![]
        }))
    })
}}

dyon_fn!{fn syntax__in_string(name: Arc<String>, text: Arc<String>) -> Variable {
    use piston_meta::syntax_errstr;

    let res = syntax_errstr(&text).map_err(|err|
        format!("When parsing meta syntax in `{}`:\n{}", name, err));
    Variable::Result(match res {
        Ok(res) => Ok(Box::new(Variable::RustObject(Arc::new(Mutex::new(Arc::new(res)))))),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(err)),
            trace: vec![]
        }))
    })
}}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn meta__syntax_in_string(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use piston_meta::Syntax;

    let text = rt.stack.pop().expect(TINVOTS);
    let text = match rt.resolve(&text) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[2].source_range(),
                        &rt.expected(x, "str"), rt))
    };
    let name = rt.stack.pop().expect(TINVOTS);
    let name = match rt.resolve(&name) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "str"), rt))
    };
    let syntax_var = rt.stack.pop().expect(TINVOTS);
    let syntax = match rt.resolve(&syntax_var) {
        &Variable::RustObject(ref obj) => obj.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "Syntax"), rt))
    };
    let res = meta::parse_syntax_data(match syntax.lock().unwrap()
        .downcast_ref::<Arc<Syntax>>() {
        Some(s) => s,
        None => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(&syntax_var, "Syntax"), rt))
    }, &name, &text);
    Ok(Some(Variable::Result(match res {
        Ok(res) => Ok(Box::new(Variable::Array(Arc::new(res)))),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(err)),
            trace: vec![]
        }))
    })))
}

dyon_fn!{fn download__url_file(url: Arc<String>, file: Arc<String>) -> Variable {
    let res = meta::download_url_to_file(&**url, &**file);
    Variable::Result(match res {
        Ok(res) => Ok(Box::new(Variable::Text(Arc::new(res)))),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(err)),
            trace: vec![]
        }))
    })
}}

#[cfg(feature = "file")]
dyon_fn!{fn save__string_file(text: Arc<String>, file: Arc<String>) -> Variable {
    use std::fs::File;
    use std::error::Error as StdError;
    use std::io::Write;

    Variable::Result(match File::create(&**file) {
        Ok(mut f) => {
            match f.write_all(text.as_bytes()) {
                Ok(_) => Ok(Box::new(Variable::Text(file))),
                Err(err) => Err(Box::new(Error {
                    message: Variable::Text(Arc::new(err.description().into())),
                    trace: vec![]
                }))
            }
        }
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(err.description().into())),
            trace: vec![]
        }))
    })
}}

#[cfg(not(feature = "file"))]
pub(crate) fn save__string_file(_: &mut Runtime) -> Result<(), String> {
    Err(FILE_SUPPORT_DISABLED.into())
}

#[cfg(feature = "file")]
dyon_fn!{fn load_string__file(file: Arc<String>) -> Variable {
    use std::fs::File;
    use std::io::Read;
    use std::error::Error as StdError;

    Variable::Result(match File::open(&**file) {
        Ok(mut f) => {
            let mut s = String::new();
            match f.read_to_string(&mut s) {
                Ok(_) => {
                    Ok(Box::new(Variable::Text(Arc::new(s))))
                }
                Err(err) => {
                    Err(Box::new(Error {
                        message: Variable::Text(Arc::new(err.description().into())),
                        trace: vec![]
                    }))
                }
            }
        }
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(err.description().into())),
            trace: vec![]
        }))
    })
}}

#[cfg(not(feature = "file"))]
pub(crate) fn load_string__file(_: &mut Runtime) -> Result<(), String> {
    Err(FILE_SUPPORT_DISABLED.into())
}

dyon_fn!{fn load_string__url(url: Arc<String>) -> Variable {
    Variable::Result(match meta::load_text_file_from_url(&**url) {
        Ok(s) => {
            Ok(Box::new(Variable::Text(Arc::new(s))))
        }
        Err(err) => {
            Err(Box::new(Error {
                message: Variable::Text(Arc::new(err)),
                trace: vec![]
            }))
        }
    })
}}

pub(crate) fn join__thread(rt: &mut Runtime) -> Result<(), String> {
    use Thread;

    let thread = rt.stack.pop().expect(TINVOTS);
    let handle_res = Thread::invalidate_handle(rt, thread);
    let v = Variable::Result({
        match handle_res {
            Ok(handle) => {
                match handle.join() {
                    Ok(res) => match res {
                        Ok(res) => Ok(Box::new(res)),
                        Err(err) => Err(Box::new(Error {
                            message: Variable::Text(Arc::new(err)),
                            trace: vec![]
                        }))
                    },
                    Err(_err) => Err(Box::new(Error {
                        message: Variable::Text(Arc::new(
                            "Thread did not exit successfully".into())),
                        trace: vec![]
                    }))
                }
            }
            Err(err) => {
                Err(Box::new(Error {
                    message: Variable::Text(Arc::new(err)),
                    trace: vec![]
                }))
            }
        }
    });
    rt.push(v);
    Ok(())
}

dyon_fn!{fn load_data__file(file: Arc<String>) -> Variable {
    use Error;

    let res = match data::load_file(&file) {
        Ok(data) => Ok(Box::new(data)),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(format!(
                        "Error loading data from file `{}`:\n{}",
                        file, err))),
            trace: vec![]
        }))
    };
    Variable::Result(res)
}}

dyon_fn!{fn load_data__string(text: Arc<String>) -> Variable {
    use Error;

    let res = match data::load_data(&text) {
        Ok(data) => Ok(Box::new(data)),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(format!(
                        "Error loading data from string `{}`:\n{}",
                        text, err))),
            trace: vec![]
        }))
    };
    Variable::Result(res)
}}

pub(crate) fn args_os(rt: &mut Runtime) -> Result<(), String> {
    let mut arr: Vec<Variable> = vec![];
    for arg in ::std::env::args_os() {
        if let Ok(t) = arg.into_string() {
            arr.push(Variable::Text(Arc::new(t)))
        } else {
            return Err("Invalid unicode in os argument".into());
        }
    }
    rt.stack.push(Variable::Array(Arc::new(arr)));
    Ok(())
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
#[cfg(feature = "file")]
pub(crate) fn save__data_file(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use std::error::Error;
    use std::fs::File;
    use std::io::BufWriter;
    use write::{write_variable, EscapeString};

    let file = rt.stack.pop().expect(TINVOTS);
    let file = match rt.resolve(&file) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "string"), rt))
    };
    let data = rt.stack.pop().expect(TINVOTS);

    let mut f = match File::create(&**file) {
        Ok(f) => BufWriter::new(f),
        Err(err) => {
            return Err(module.error(call.args[0].source_range(),
                       &format!("{}\nError when creating file `{}`:\n{}",
                        rt.stack_trace(), file, err.description()), rt))
        }
    };
    let res = match write_variable(&mut f, rt, &data, EscapeString::Json, 0) {
        Ok(()) => Ok(Box::new(Variable::Text(file.clone()))),
        Err(err) => {
            Err(Box::new(::Error {
                message: Variable::Text(Arc::new(format!(
                            "Error when writing to file `{}`:\n{}",
                            file, err.description()))),
                trace: vec![]
            }))
        }
    };
    Ok(Some(Variable::Result(res)))
}

#[cfg(not(feature = "file"))]
pub(crate) fn save__data_file(
    _: &mut Runtime,
    _: &ast::Call,
    _: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    Err(FILE_SUPPORT_DISABLED.into())
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn json_from_meta_data(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use std::error::Error;

    let meta_data = rt.stack.pop().expect(TINVOTS);
    let json = match rt.resolve(&meta_data) {
        &Variable::Array(ref arr) => {
            try!(meta::json_from_meta_data(arr).map_err(|err| {
                format!("{}\nError when generating JSON:\n{}",
                        rt.stack_trace(),
                        err.description())
            }))
        }
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "array"), rt))
    };
    Ok(Some(Variable::Text(Arc::new(json))))
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn errstr__string_start_len_msg(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use piston_meta::ParseErrorHandler;
    use range::Range;

    let msg = rt.stack.pop().expect(TINVOTS);
    let msg = match rt.resolve(&msg) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[3].source_range(),
                        &rt.expected(x, "str"), rt))
    };
    let len = rt.stack.pop().expect(TINVOTS);
    let len = match rt.resolve(&len) {
        &Variable::F64(v, _) => v as usize,
        x => return Err(module.error(call.args[2].source_range(),
                        &rt.expected(x, "f64"), rt))
    };
    let start = rt.stack.pop().expect(TINVOTS);
    let start = match rt.resolve(&start) {
        &Variable::F64(v, _) => v as usize,
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "f64"), rt))
    };
    let source = rt.stack.pop().expect(TINVOTS);
    let source = match rt.resolve(&source) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "str"), rt))
    };

    let mut buf: Vec<u8> = vec![];
    ParseErrorHandler::new(&source)
        .write_msg(&mut buf, Range::new(start, len), &msg)
        .unwrap();
    Ok(Some(Variable::Text(Arc::new(String::from_utf8(buf).unwrap()))))
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn has(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let key = rt.stack.pop().expect(TINVOTS);
    let key = match rt.resolve(&key) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "str"), rt))
    };
    let obj = rt.stack.pop().expect(TINVOTS);
    let res = match rt.resolve(&obj) {
        &Variable::Object(ref obj) => obj.contains_key(&key),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "object"), rt))
    };
    Ok(Some(Variable::bool(res)))
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn keys(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let obj = rt.stack.pop().expect(TINVOTS);
    let res = Variable::Array(Arc::new(match rt.resolve(&obj) {
        &Variable::Object(ref obj) => {
            obj.keys().map(|k| Variable::Text(k.clone())).collect()
        }
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "object"), rt))
    }));
    Ok(Some(res))
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn chars(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let t = rt.stack.pop().expect(TINVOTS);
    let t = match rt.resolve(&t) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "str"), rt))
    };
    let res = t.chars()
        .map(|ch| {
            let mut s = String::new();
            s.push(ch);
            Variable::Text(Arc::new(s))
        })
        .collect::<Vec<_>>();
    Ok(Some(Variable::Array(Arc::new(res))))
}

dyon_fn!{fn now() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(val) => val.as_secs() as f64 +
                   val.subsec_nanos() as f64 / 1.0e9,
        Err(err) => -{
            let val = err.duration();
            val.as_secs() as f64 +
            val.subsec_nanos() as f64 / 1.0e9
        }
    }
}}

dyon_fn!{fn is_nan(v: f64) -> bool {v.is_nan()}}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn wait_next(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>
) -> Result<Option<Variable>, String> {
    use std::error::Error;

    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::In(ref mutex) => {
            match mutex.lock() {
                Ok(x) => match x.recv() {
                    Ok(x) => Variable::Option(Some(Box::new(x))),
                    Err(_) => Variable::Option(None),
                },
                Err(err) => {
                    return Err(module.error(call.source_range,
                    &format!("Can not lock In mutex:\n{}", err.description()), rt));
                }
            }
        }
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "in"), rt))
    }))
}

// TODO: Can't be rewritten as an external function because it reports errors on arguments.
pub(crate) fn next(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>
) -> Result<Option<Variable>, String> {
    use std::error::Error;

    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::In(ref mutex) => {
            match mutex.lock() {
                Ok(x) => match x.try_recv() {
                    Ok(x) => Variable::Option(Some(Box::new(x))),
                    Err(_) => Variable::Option(None),
                },
                Err(err) => {
                    return Err(module.error(call.source_range,
                    &format!("Can not lock In mutex:\n{}", err.description()), rt));
                }
            }
        }
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "in"), rt))
    }))
}
