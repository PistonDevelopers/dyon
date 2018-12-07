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

pub(crate) fn x(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Vec4(ref vec4) => Variable::f64(vec4[0] as f64),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "number"), rt))
    }))
}

pub(crate) fn y(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Vec4(ref vec4) => Variable::f64(vec4[1] as f64),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "number"), rt))
    }))
}

pub(crate) fn z(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Vec4(ref vec4) => Variable::f64(vec4[2] as f64),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "number"), rt))
    }))
}

pub(crate) fn w(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Vec4(ref vec4) => Variable::f64(vec4[3] as f64),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "number"), rt))
    }))
}

pub(crate) fn s(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let ind = rt.stack.pop().expect(TINVOTS);
    let ind = match rt.resolve(&ind) {
        &Variable::F64(val, _) => val,
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "number"), rt))
    };
    let v = rt.stack.pop().expect(TINVOTS);
    let s = match rt.resolve(&v) {
        &Variable::Vec4(ref v) => {
            match v.get(ind as usize) {
                Some(&s) => s as f64,
                None => {
                    return Err(module.error(call.source_range,
                        &format!("{}\nIndex out of bounds `{}`",
                            rt.stack_trace(), ind), rt))
                }
            }
        }
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "vec4"), rt))
    };
    Ok(Some(Variable::f64(s)))
}

pub(crate) fn rx(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Mat4(ref m) => Variable::Vec4([m[0][0], m[1][0], m[2][0], m[3][0]]),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "mat4"), rt))
    }))
}

pub(crate) fn ry(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Mat4(ref m) => Variable::Vec4([m[0][1], m[1][1], m[2][1], m[3][1]]),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "mat4"), rt))
    }))
}

pub(crate) fn rz(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Mat4(ref m) => Variable::Vec4([m[0][2], m[1][2], m[2][2], m[3][2]]),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "mat4"), rt))
    }))
}

pub(crate) fn rw(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Mat4(ref m) => Variable::Vec4([m[0][3], m[1][3], m[2][3], m[3][3]]),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "mat4"), rt))
    }))
}

pub(crate) fn rv(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let ind = rt.stack.pop().expect(TINVOTS);
    let ind = match rt.resolve(&ind) {
        &Variable::F64(val, _) => val,
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "number"), rt))
    } as usize;
    if ind >= 4 {
        return Err(module.error(call.source_range,
            &format!("{}\nIndex out of bounds `{}`",
                rt.stack_trace(), ind), rt))
    }
    let m = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&m) {
        &Variable::Mat4(ref m) => [m[0][ind], m[1][ind], m[2][ind], m[3][ind]],
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "mat4"), rt))
    };
    Ok(Some(Variable::Vec4(v)))
}

pub(crate) fn cx(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Mat4(ref m) => Variable::Vec4(m[0]),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "mat4"), rt))
    }))
}

pub(crate) fn cy(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Mat4(ref m) => Variable::Vec4(m[1]),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "mat4"), rt))
    }))
}

pub(crate) fn cz(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Mat4(ref m) => Variable::Vec4(m[2]),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "mat4"), rt))
    }))
}

pub(crate) fn cw(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Mat4(ref m) => Variable::Vec4(m[3]),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "mat4"), rt))
    }))
}

pub(crate) fn cv(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let ind = rt.stack.pop().expect(TINVOTS);
    let ind = match rt.resolve(&ind) {
        &Variable::F64(val, _) => val,
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "number"), rt))
    } as usize;
    if ind >= 4 {
        return Err(module.error(call.source_range,
            &format!("{}\nIndex out of bounds `{}`",
                rt.stack_trace(), ind), rt))
    }
    let m = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&m) {
        &Variable::Mat4(ref m) => m[ind],
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "mat4"), rt))
    };
    Ok(Some(Variable::Vec4(v)))
}

pub(crate) fn clone(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(rt.resolve(&v).deep_clone(&rt.stack)))
}

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

pub(crate) fn println(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use write::{print_variable, EscapeString};

    let x = rt.stack.pop().expect(TINVOTS);
    print_variable(rt, &x, EscapeString::None);
    println!("");
    Ok(None)
}

pub(crate) fn print(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use write::{print_variable, EscapeString};

    let x = rt.stack.pop().expect(TINVOTS);
    print_variable(rt, &x, EscapeString::None);
    Ok(None)
}

pub(crate) fn sqrt(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.sqrt())
}

pub(crate) fn sin(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.sin())
}

pub(crate) fn asin(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.asin())
}

pub(crate) fn cos(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.cos())
}

pub(crate) fn acos(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.acos())
}

pub(crate) fn tan(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.tan())
}

pub(crate) fn atan(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.atan())
}

pub(crate) fn atan2(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let x = rt.stack.pop().expect(TINVOTS);
    let x = match rt.resolve(&x) {
        &Variable::F64(b, _) => b,
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "number"), rt))
    };
    let y = rt.stack.pop().expect(TINVOTS);
    let y = match rt.resolve(&y) {
        &Variable::F64(b, _) => b,
        y => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(y, "number"), rt))
    };
    Ok(Some(Variable::f64(y.atan2(x))))
}

pub(crate) fn exp(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.exp())
}

pub(crate) fn ln(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.ln())
}

pub(crate) fn log2(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.log2())
}

pub(crate) fn log10(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.log10())
}

pub(crate) fn round(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.round())
}

pub(crate) fn abs(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.abs())
}

pub(crate) fn floor(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.floor())
}

pub(crate) fn ceil(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    rt.unary_f64(call, module, |a| a.ceil())
}

pub(crate) fn sleep(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use std::thread::sleep;
    use std::time::Duration;

    let v = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&v) {
        &Variable::F64(b, _) => b,
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "number"), rt))
    };
    let secs = v as u64;
    let nanos = (v.fract() * 1.0e9) as u32;
    sleep(Duration::new(secs, nanos));
    Ok(None)
}

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

pub(crate) fn random(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use rand::Rng;

    Ok(Some(Variable::f64(rt.rng.gen())))
}

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

pub(crate) fn read_line(
    _rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use std::io::{self, Write};

    let mut input = String::new();
    io::stdout().flush().unwrap();
    let error = match io::stdin().read_line(&mut input) {
        Ok(_) => None,
        Err(error) => Some(error)
    };
    let v = if let Some(error) = error {
        // TODO: Return error instead.
        Variable::RustObject(
            Arc::new(Mutex::new(error)))
    } else {
        Variable::Text(Arc::new(input))
    };
    Ok(Some(v))
}

pub(crate) fn read_number(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use std::io::{self, Write};

    let err = rt.stack.pop().expect(TINVOTS);
    let err = match rt.resolve(&err) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "text"), rt))
    };
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();
    let mut rv: Option<Variable> = None;
    loop {
        input.clear();
        stdout.flush().unwrap();
        match stdin.read_line(&mut input) {
            Ok(_) => {}
            Err(error) => {
                // TODO: Return error instead.
                rt.stack.push(Variable::RustObject(
                    Arc::new(Mutex::new(error))));
                break;
            }
        };
        match input.trim().parse::<f64>() {
            Ok(v) => {
                rv = Some(Variable::f64(v));
                break;
            }
            Err(_) => {
                println!("{}", err);
            }
        }
    }
    Ok(rv)
}

pub(crate) fn parse_number(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let text = rt.stack.pop().expect(TINVOTS);
    let text= match rt.resolve(&text) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "text"), rt))
    };
    Ok(Some(Variable::Option(match text.trim().parse::<f64>() {
        Ok(v) => Some(Box::new(Variable::f64(v))),
        Err(_) => None
    })))
}

pub(crate) fn trim(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&v) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "text"), rt))
    };
    let v = Variable::Text(Arc::new(v.trim().into()));
    Ok(Some(v))
}

pub(crate) fn trim_left(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&v) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "text"), rt))
    };
    let v = Variable::Text(Arc::new(v.trim_left().into()));
    Ok(Some(v))
}

pub(crate) fn trim_right(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let mut v = match rt.resolve(&v) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "text"), rt))
    };
    {
        let w = Arc::make_mut(&mut v);
        while let Some(ch) = w.pop() {
            if !ch.is_whitespace() { w.push(ch); break; }
        }
    }
    Ok(Some(Variable::Text(v)))
}

pub(crate) fn _str(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use write::{write_variable, EscapeString};

    let v = rt.stack.pop().expect(TINVOTS);
    let mut buf: Vec<u8> = vec![];
    write_variable(&mut buf, rt, rt.resolve(&v), EscapeString::None, 0).unwrap();
    let v = Variable::Text(Arc::new(String::from_utf8(buf).unwrap()));
    Ok(Some(v))
}

pub(crate) fn json_string(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use write::{write_variable, EscapeString};

    let v = rt.stack.pop().expect(TINVOTS);
    let mut buf: Vec<u8> = vec![];
    write_variable(&mut buf, rt, rt.resolve(&v), EscapeString::Json, 0).unwrap();
    Ok(Some(Variable::Text(Arc::new(String::from_utf8(buf).unwrap()))))
}

pub(crate) fn str__color(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&v) {
        &Variable::Vec4(val) => val,
        x => return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "vec4"), rt))
    };
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
    Ok(Some(Variable::Text(Arc::new(String::from_utf8(buf).unwrap()))))
}

pub(crate) fn srgb_to_linear__color(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&v) {
        &Variable::Vec4(val) => val,
        x => return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "vec4"), rt))
    };
    let to_linear = |f: f32| {
        if f <= 0.04045 {
            f / 12.92
        } else {
            ((f + 0.055) / 1.055).powf(2.4)
        }
    };
    Ok(Some(Variable::Vec4(
        [to_linear(v[0]), to_linear(v[1]), to_linear(v[2]), v[3]]
    )))
}

pub(crate) fn linear_to_srgb__color(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&v) {
        &Variable::Vec4(val) => val,
        x => return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "vec4"), rt))
    };
    let to_srgb = |f: f32| {
        if f <= 0.0031308 {
            f * 12.92
        } else {
            1.055 * f.powf(1.0 / 2.4) - 0.055
        }
    };
    Ok(Some(Variable::Vec4(
        [to_srgb(v[0]), to_srgb(v[1]), to_srgb(v[2]), v[3]]
    )))
}

pub(crate) fn _typeof(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::Text(_) => rt.text_type.clone(),
        &Variable::F64(_, _) => rt.f64_type.clone(),
        &Variable::Vec4(_) => rt.vec4_type.clone(),
        &Variable::Mat4(_) => rt.mat4_type.clone(),
        &Variable::Return => rt.return_type.clone(),
        &Variable::Bool(_, _) => rt.bool_type.clone(),
        &Variable::Object(_) => rt.object_type.clone(),
        &Variable::Array(_) => rt.array_type.clone(),
        &Variable::Link(_) => rt.link_type.clone(),
        &Variable::Ref(_) => rt.ref_type.clone(),
        &Variable::UnsafeRef(_) => rt.unsafe_ref_type.clone(),
        &Variable::RustObject(_) => rt.rust_object_type.clone(),
        &Variable::Option(_) => rt.option_type.clone(),
        &Variable::Result(_) => rt.result_type.clone(),
        &Variable::Thread(_) => rt.thread_type.clone(),
        &Variable::Closure(_, _) => rt.closure_type.clone(),
        &Variable::In(_) => rt.in_type.clone(),
    }))
}

pub(crate) fn debug(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    println!("Stack {:#?}", rt.stack);
    println!("Locals {:#?}", rt.local_stack);
    println!("Currents {:#?}", rt.current_stack);
    Ok(None)
}

pub(crate) fn backtrace(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    println!("{:#?}", rt.call_stack);
    Ok(None)
}

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

pub(crate) fn functions(
    _rt: &mut Runtime,
    _call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    // List available functions in scope.
    let v = Variable::Array(Arc::new(functions::list_functions(module)));
    Ok(Some(v))
}

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

pub(crate) fn none(
    _rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    Ok(Some(Variable::Option(None)))
}

pub(crate) fn some(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(Variable::Option(Some(Box::new(
        rt.resolve(&v).deep_clone(&rt.stack)
    )))))
}

pub(crate) fn ok(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(Variable::Result(Ok(Box::new(
        rt.resolve(&v).deep_clone(&rt.stack)
    )))))
}

pub(crate) fn err(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(Variable::Result(Err(Box::new(
        Error {
            message: rt.resolve(&v).deep_clone(&rt.stack),
            trace: vec![]
        })))))
}

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

pub(crate) fn dir__angle(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    Ok(Some(match rt.resolve(&v) {
        &Variable::F64(val, _) => Variable::Vec4([val.cos() as f32, val.sin() as f32, 0.0, 0.0]),
        x => {
            return Err(module.error(call.args[0].source_range(),
                &rt.expected(x, "err(_)"), rt));
        }
    }))
}

pub(crate) fn load__meta_file(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let file = rt.stack.pop().expect(TINVOTS);
    let meta = rt.stack.pop().expect(TINVOTS);
    let file = match rt.resolve(&file) {
        &Variable::Text(ref file) => file.clone(),
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "str"), rt))
    };
    let meta = match rt.resolve(&meta) {
        &Variable::Text(ref meta) => meta.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "str"), rt))
    };
    let res = meta::load_meta_file(&**meta, &**file);
    Ok(Some(Variable::Result(match res {
        Ok(res) => Ok(Box::new(Variable::Array(Arc::new(res)))),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(err)),
            trace: vec![]
        }))
    })))
}

pub(crate) fn load__meta_url(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let url = rt.stack.pop().expect(TINVOTS);
    let meta = rt.stack.pop().expect(TINVOTS);
    let url = match rt.resolve(&url) {
        &Variable::Text(ref url) => url.clone(),
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "str"), rt))
    };
    let meta = match rt.resolve(&meta) {
        &Variable::Text(ref meta) => meta.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "str"), rt))
    };
    let res = meta::load_meta_url(&**meta, &**url);
    Ok(Some(Variable::Result(match res {
        Ok(res) => Ok(Box::new(Variable::Array(Arc::new(res)))),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(err)),
            trace: vec![]
        }))
    })))
}

pub(crate) fn syntax__in_string(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use piston_meta::syntax_errstr;

    let text = rt.stack.pop().expect(TINVOTS);
    let text = match rt.resolve(&text) {
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
    let res = syntax_errstr(&text).map_err(|err|
        format!("When parsing meta syntax in `{}`:\n{}", name, err));
    Ok(Some(Variable::Result(match res {
        Ok(res) => Ok(Box::new(Variable::RustObject(Arc::new(Mutex::new(Arc::new(res)))))),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(err)),
            trace: vec![]
        }))
    })))
}

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

pub(crate) fn download__url_file(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let file = rt.stack.pop().expect(TINVOTS);
    let url = rt.stack.pop().expect(TINVOTS);
    let file = match rt.resolve(&file) {
        &Variable::Text(ref file) => file.clone(),
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "str"), rt))
    };
    let url = match rt.resolve(&url) {
        &Variable::Text(ref url) => url.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "str"), rt))
    };

    let res = meta::download_url_to_file(&**url, &**file);
    Ok(Some(Variable::Result(match res {
        Ok(res) => Ok(Box::new(Variable::Text(Arc::new(res)))),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(err)),
            trace: vec![]
        }))
    })))
}

#[cfg(feature = "file")]
pub(crate) fn save__string_file(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use std::fs::File;
    use std::io::Write;
    use std::error::Error as StdError;

    let file = rt.stack.pop().expect(TINVOTS);
    let text = rt.stack.pop().expect(TINVOTS);
    let file = match rt.resolve(&file) {
        &Variable::Text(ref file) => file.clone(),
        x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "str"), rt))
    };
    let text = match rt.resolve(&text) {
        &Variable::Text(ref text) => text.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "str"), rt))
    };

    Ok(Some(Variable::Result(match File::create(&**file) {
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
    })))
}

#[cfg(not(feature = "file"))]
pub(crate) fn save__string_file(
    _: &mut Runtime,
    _: &ast::Call,
    _: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    Err(FILE_SUPPORT_DISABLED.into())
}

#[cfg(feature = "file")]
pub(crate) fn load_string__file(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use std::fs::File;
    use std::io::Read;
    use std::error::Error as StdError;

    let file = rt.stack.pop().expect(TINVOTS);
    let file = match rt.resolve(&file) {
        &Variable::Text(ref file) => file.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "str"), rt))
    };

    Ok(Some(Variable::Result(match File::open(&**file) {
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
    })))
}

#[cfg(not(feature = "file"))]
pub(crate) fn load_string__file(
    _: &mut Runtime,
    _: &ast::Call,
    _: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    Err(FILE_SUPPORT_DISABLED.into())
}

pub(crate) fn load_string__url(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let url = rt.stack.pop().expect(TINVOTS);
    let url = match rt.resolve(&url) {
        &Variable::Text(ref url) => url.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "str"), rt))
    };

    Ok(Some(Variable::Result(match meta::load_text_file_from_url(&**url) {
        Ok(s) => {
            Ok(Box::new(Variable::Text(Arc::new(s))))
        }
        Err(err) => {
            Err(Box::new(Error {
                message: Variable::Text(Arc::new(err)),
                trace: vec![]
            }))
        }
    })))
}

pub(crate) fn join__thread(
    rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
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
    Ok(Some(v))
}

pub(crate) fn load_data__file(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use Error;

    let file = rt.stack.pop().expect(TINVOTS);
    let file = match rt.resolve(&file) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "string"), rt))
    };
    let res = match data::load_file(&file) {
        Ok(data) => Ok(Box::new(data)),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(format!(
                        "Error loading data from file `{}`:\n{}",
                        file, err))),
            trace: vec![]
        }))
    };
    Ok(Some(Variable::Result(res)))
}

pub(crate) fn load_data__string(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use Error;

    let text = rt.stack.pop().expect(TINVOTS);
    let text = match rt.resolve(&text) {
        &Variable::Text(ref t) => t.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "string"), rt))
    };
    let res = match data::load_data(&text) {
        Ok(data) => Ok(Box::new(data)),
        Err(err) => Err(Box::new(Error {
            message: Variable::Text(Arc::new(format!(
                        "Error loading data from string `{}`:\n{}",
                        text, err))),
            trace: vec![]
        }))
    };
    Ok(Some(Variable::Result(res)))
}

pub(crate) fn args_os(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>
) -> Result<Option<Variable>, String> {
    let mut arr: Vec<Variable> = vec![];
    for arg in ::std::env::args_os() {
        if let Ok(t) = arg.into_string() {
            arr.push(Variable::Text(Arc::new(t)))
        } else {
            return Err(module.error(call.source_range,
                      "Invalid unicode in os argument", rt));
        }
    }
    Ok(Some(Variable::Array(Arc::new(arr))))
}

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

pub(crate) fn now(
    _rt: &mut Runtime,
    _call: &ast::Call,
    _module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let val = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(val) => Variable::f64(val.as_secs() as f64 +
                                 val.subsec_nanos() as f64 / 1.0e9),
        Err(err) => Variable::f64(-{
            let val = err.duration();
            val.as_secs() as f64 +
            val.subsec_nanos() as f64 / 1.0e9
        })
    };
    Ok(Some(val))
}

pub(crate) fn is_nan(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Arc<Module>,
) -> Result<Option<Variable>, String> {
    let v = rt.stack.pop().expect(TINVOTS);
    let v = match rt.resolve(&v) {
        &Variable::F64(ref v, _) => v.clone(),
        x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "number"), rt))
    };
    Ok(Some(Variable::bool(v.is_nan())))
}

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
