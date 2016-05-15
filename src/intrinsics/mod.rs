use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io;
use rand::Rng;
use piston_meta::json;

use runtime::{Expect, Flow, Runtime, Side};
use ast;
use prelude::{Lt, PreludeFunction};
use Type;

use Variable;
use Module;
use Error;

mod meta;

const TINVOTS: &'static str = "There is no value on the stack";

pub fn standard(f: &mut HashMap<Arc<String>, PreludeFunction>) {
    let sarg = |f: &mut HashMap<Arc<String>, PreludeFunction>, name: &str, ty: Type, ret: Type| {
        f.insert(Arc::new(name.into()), PreludeFunction {
            lts: vec![Lt::Default],
            tys: vec![ty],
            ret: ret
        });
    };

    sarg(f, "println", Type::Any, Type::Void);
    sarg(f, "print", Type::Any, Type::Void);
    sarg(f, "clone", Type::Any, Type::Any);
    f.insert(Arc::new("debug".into()), PreludeFunction {
        lts: vec![],
        tys: vec![],
        ret: Type::Void
    });
    f.insert(Arc::new("backtrace".into()), PreludeFunction {
        lts: vec![],
        tys: vec![],
        ret: Type::Void
    });
    sarg(f, "sleep", Type::F64, Type::Void);
    f.insert(Arc::new("random".into()), PreludeFunction {
        lts: vec![],
        tys: vec![],
        ret: Type::F64
    });
    sarg(f, "read_number", Type::Text, Type::Any);
    f.insert(Arc::new("read_line".into()), PreludeFunction {
        lts: vec![],
        tys: vec![],
        ret: Type::Any
    });
    sarg(f, "len", Type::array(), Type::F64);
    f.insert(Arc::new("push_ref(mut,_)".into()), PreludeFunction {
        lts: vec![Lt::Default, Lt::Arg(0)],
        tys: vec![Type::array(), Type::Any],
        ret: Type::F64
    });
    f.insert(Arc::new("push(mut,_)".into()), PreludeFunction {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::array(), Type::Any],
        ret: Type::Void
    });
    f.insert(Arc::new("pop(mut)".into()), PreludeFunction {
        lts: vec![Lt::Return],
        tys: vec![Type::array()],
        ret: Type::Any
    });
    sarg(f, "trim", Type::Text, Type::Text);
    sarg(f, "trim_left", Type::Text, Type::Text);
    sarg(f, "trim_right", Type::Text, Type::Text);
    sarg(f, "to_string", Type::Any, Type::Text);
    sarg(f, "to_string_color", Type::Vec4, Type::Text);
    sarg(f, "srgb_to_linear_color", Type::Vec4, Type::Vec4);
    sarg(f, "linear_to_srgb_color", Type::Vec4, Type::Vec4);
    sarg(f, "typeof", Type::Any, Type::Text);
    sarg(f, "round", Type::F64, Type::F64);
    sarg(f, "abs", Type::F64, Type::F64);
    sarg(f, "floor", Type::F64, Type::F64);
    sarg(f, "ceil", Type::F64, Type::F64);
    sarg(f, "sqrt", Type::F64, Type::F64);
    sarg(f, "sin", Type::F64, Type::F64);
    sarg(f, "asin", Type::F64, Type::F64);
    sarg(f, "cos", Type::F64, Type::F64);
    sarg(f, "acos", Type::F64, Type::F64);
    sarg(f, "tan", Type::F64, Type::F64);
    sarg(f, "atan", Type::F64, Type::F64);
    sarg(f, "exp", Type::F64, Type::F64);
    sarg(f, "ln", Type::F64, Type::F64);
    sarg(f, "log2", Type::F64, Type::F64);
    sarg(f, "log10", Type::F64, Type::F64);
    sarg(f, "load", Type::Text, Type::result());
    f.insert(Arc::new("load_source_imports".into()), PreludeFunction {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Text, Type::array()],
        ret: Type::result()
    });
    f.insert(Arc::new("call".into()), PreludeFunction {
        lts: vec![Lt::Default; 3],
        tys: vec![Type::Any, Type::Text, Type::array()],
        ret: Type::Void
    });
    f.insert(Arc::new("call_ret".into()), PreludeFunction {
        lts: vec![Lt::Default; 3],
        tys: vec![Type::Any, Type::Text, Type::array()],
        ret: Type::Any
    });
    f.insert(Arc::new("functions".into()), PreludeFunction {
        lts: vec![],
        tys: vec![],
        ret: Type::Any
    });
    f.insert(Arc::new("none".into()), PreludeFunction {
        lts: vec![],
        tys: vec![],
        ret: Type::option()
    });
    sarg(f, "unwrap", Type::Any, Type::Any);
    sarg(f, "unwrap_err", Type::Any, Type::Any);
    sarg(f, "some", Type::Any, Type::option());
    sarg(f, "ok", Type::Any, Type::result());
    sarg(f, "err", Type::Any, Type::result());
    sarg(f, "is_err", Type::Any, Type::Bool);
    sarg(f, "x", Type::Vec4, Type::F64);
    sarg(f, "y", Type::Vec4, Type::F64);
    sarg(f, "z", Type::Vec4, Type::F64);
    sarg(f, "w", Type::Vec4, Type::F64);
    sarg(f, "dir_angle", Type::F64, Type::Vec4);
    f.insert(Arc::new("load_meta_file".into()), PreludeFunction {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Text; 2],
        ret: Type::Result(Box::new(Type::array()))
    });
    f.insert(Arc::new("load_meta_url".into()), PreludeFunction {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Text; 2],
        ret: Type::Result(Box::new(Type::array()))
    });
    sarg(f, "join_thread", Type::thread(), Type::Result(Box::new(Type::Any)));
}

enum EscapeString {
    Json,
    None
}


fn write_variable<W>(
    w: &mut W,
    rt: &Runtime,
    v: &Variable,
    escape_string: EscapeString
) -> Result<(), io::Error>
    where W: io::Write
{
    match *rt.resolve(v) {
        Variable::Text(ref t) => {
            match escape_string {
                EscapeString::Json => {
                    try!(json::write_string(w, t));
                }
                EscapeString::None => {
                    try!(write!(w, "{}", t))
                }
            }
        }
        Variable::F64(x) => {
            try!(write!(w, "{}", x));
        }
        Variable::Vec4(v) => {
            try!(write!(w, "({}, {}", v[0], v[1]));
            if v[2] != 0.0 || v[3] != 0.0 {
                try!(write!(w, ", {}", v[2]));
                if v[3] != 0.0 {
                    try!(write!(w, ", {})", v[3]));
                } else {
                    try!(write!(w, ")"));
                }
            } else {
                try!(write!(w, ")"));
            }
        }
        Variable::Bool(x) => {
            try!(write!(w, "{}", x));
        }
        Variable::Ref(ind) => {
            print_variable(rt, &rt.stack[ind], escape_string);
        }
        Variable::Object(ref obj) => {
            try!(write!(w, "{{"));
            let n = obj.len();
            for (i, (k, v)) in obj.iter().enumerate() {
                try!(write!(w, "{}: ", k));
                try!(write_variable(w, rt, v, EscapeString::Json));
                if i + 1 < n {
                    try!(write!(w, ", "));
                }
            }
            try!(write!(w, "}}"));
        }
        Variable::Array(ref arr) => {
            try!(write!(w, "["));
            let n = arr.len();
            for (i, v) in arr.iter().enumerate() {
                try!(write_variable(w, rt, v, EscapeString::Json));
                if i + 1 < n {
                    try!(write!(w, ", "));
                }
            }
            try!(write!(w, "]"));
        }
        Variable::Option(ref opt) => {
            match opt {
                &None => {
                    try!(write!(w, "none()"))
                }
                &Some(ref v) => {
                    try!(write!(w, "some("));
                    try!(write_variable(w, rt, v, EscapeString::Json));
                    try!(write!(w, ")"));
                }
            }
        }
        Variable::Result(ref res) => {
            match res {
                &Err(ref err) => {
                    try!(write!(w, "err("));
                    try!(write_variable(w, rt, &err.message,
                                        EscapeString::Json));
                    try!(write!(w, ")"));
                }
                &Ok(ref ok) => {
                    try!(write!(w, "ok("));
                    try!(write_variable(w, rt, ok, EscapeString::Json));
                    try!(write!(w, ")"));
                }
            }
        }
        Variable::Thread(_) => try!(write!(w, "_thread")),
        Variable::Return => try!(write!(w, "_return")),
        Variable::UnsafeRef(_) => try!(write!(w, "_unsafe_ref")),
        Variable::RustObject(_) => try!(write!(w, "_rust_object")),
        // ref x => panic!("Could not print out `{:?}`", x)
    }
    Ok(())
}

fn print_variable(rt: &Runtime, v: &Variable, escape_string: EscapeString) {
    write_variable(&mut io::stdout(), rt, v, escape_string).unwrap();
}

pub fn call_standard(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Module
) -> Result<(Expect, Flow), String> {
    let st = rt.stack.len();
    let lc = rt.local_stack.len();
    for arg in &call.args {
        match try!(rt.expression(arg, Side::Right, module)) {
            (x, Flow::Return) => { return Ok((x, Flow::Return)); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(arg.source_range(),
                    &format!("{}\nExpected something. \
                    Expression did not return a value.",
                    rt.stack_trace())))
        };
    }
    let vec4_comp = |rt: &mut Runtime, module: &Module, call: &ast::Call, i: usize|
                     -> Result<Expect, String> {
        rt.push_fn(call.name.clone(), None, st + 1, lc);
        let v = rt.stack.pop().expect(TINVOTS);
        let v = match rt.resolve(&v) {
            &Variable::Vec4(ref vec4) => Variable::F64(vec4[i] as f64),
            x => return Err(module.error(call.args[i].source_range(),
                            &rt.expected(x, "number")))
        };
        rt.stack.push(v);
        rt.pop_fn(call.name.clone());
        Ok(Expect::Something)
    };
    let expect = match &**call.name {
        "x" => try!(vec4_comp(rt, module, call, 0)),
        "y" => try!(vec4_comp(rt, module, call, 1)),
        "z" => try!(vec4_comp(rt, module, call, 2)),
        "w" => try!(vec4_comp(rt, module, call, 3)),
        "clone" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = rt.resolve(&v).deep_clone(&rt.stack);
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "println" => {
            rt.push_fn(call.name.clone(), None, st, lc);
            let x = rt.stack.pop().expect(TINVOTS);
            print_variable(rt, &x, EscapeString::None);
            println!("");
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "print" => {
            rt.push_fn(call.name.clone(), None, st, lc);
            let x = rt.stack.pop().expect(TINVOTS);
            print_variable(rt, &x, EscapeString::None);
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "sqrt" => try!(rt.unary_f64(call, module, |a| a.sqrt())),
        "sin" => try!(rt.unary_f64(call, module, |a| a.sin())),
        "asin" => try!(rt.unary_f64(call, module, |a| a.asin())),
        "cos" => try!(rt.unary_f64(call, module, |a| a.cos())),
        "acos" => try!(rt.unary_f64(call, module, |a| a.acos())),
        "tan" => try!(rt.unary_f64(call, module, |a| a.tan())),
        "atan" => try!(rt.unary_f64(call, module, |a| a.atan())),
        "exp" => try!(rt.unary_f64(call, module, |a| a.exp())),
        "ln" => try!(rt.unary_f64(call, module, |a| a.ln())),
        "log2" => try!(rt.unary_f64(call, module, |a| a.log2())),
        "log10" => try!(rt.unary_f64(call, module, |a| a.log10())),
        "round" => try!(rt.unary_f64(call, module, |a| a.round())),
        "abs" => try!(rt.unary_f64(call, module, |a| a.abs())),
        "floor" => try!(rt.unary_f64(call, module, |a| a.floor())),
        "ceil" => try!(rt.unary_f64(call, module, |a| a.ceil())),
        "sleep" => {
            use std::thread::sleep;
            use std::time::Duration;

            rt.push_fn(call.name.clone(), None, st, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::F64(b) => b,
                x => return Err(module.error(call.args[0].source_range(),
                                &rt.expected(x, "number")))
            };
            let secs = v as u64;
            let nanos = (v.fract() * 1.0e9) as u32;
            sleep(Duration::new(secs, nanos));
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "random" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = Variable::F64(rt.rng.gen());
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "len" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = match rt.stack.pop() {
                Some(v) => v,
                None => panic!(TINVOTS)
            };

            let v = {
                let arr = match rt.resolve(&v) {
                    &Variable::Array(ref arr) => arr,
                    x => return Err(module.error(call.args[0].source_range(),
                                    &rt.expected(x, "array")))
                };
                Variable::F64(arr.len() as f64)
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "push_ref(mut,_)" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
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
                            rt.stack_trace())));
                }
            } else {
                return Err(module.error(call.args[0].source_range(),
                    &format!("{}\nExpected reference to array",
                        rt.stack_trace())));
            }
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "push(mut,_)" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
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
                            rt.stack_trace())));
                }
            } else {
                return Err(module.error(call.args[0].source_range(),
                    &format!("{}\nExpected reference to array",
                        rt.stack_trace())));
            }
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "pop(mut)" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
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
                            rt.stack_trace())));
                }
            } else {
                return Err(module.error(call.args[0].source_range(),
                    &format!("{}\nExpected reference to array",
                        rt.stack_trace())));
            }
            match v {
                None => return Err(module.error(call.args[0].source_range(),
                    &format!("{}\nExpected non-empty array",
                        rt.stack_trace()))),
                Some(val) => {
                    rt.stack.push(val);
                }
            }
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "read_line" => {
            use std::io::{self, Write};

            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let mut input = String::new();
            io::stdout().flush().unwrap();
            let error = match io::stdin().read_line(&mut input) {
                Ok(_) => None,
                Err(error) => Some(error)
            };
            if let Some(error) = error {
                rt.stack.push(Variable::RustObject(
                    Arc::new(Mutex::new(error))));
            } else {
                rt.stack.push(Variable::Text(Arc::new(input)));
            }
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "read_number" => {
            use std::io::{self, Write};

            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let err = rt.stack.pop().expect(TINVOTS);
            let err = match rt.resolve(&err) {
                &Variable::Text(ref t) => t.clone(),
                x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "text")))
            };
            let stdin = io::stdin();
            let mut stdout = io::stdout();
            let mut input = String::new();
            loop {
                input.clear();
                stdout.flush().unwrap();
                match stdin.read_line(&mut input) {
                    Ok(_) => {}
                    Err(error) => {
                        rt.stack.push(Variable::RustObject(
                            Arc::new(Mutex::new(error))));
                        break;
                    }
                };
                match input.trim().parse::<f64>() {
                    Ok(v) => {
                        rt.stack.push(Variable::F64(v));
                        break;
                    }
                    Err(_) => {
                        println!("{}", err);
                    }
                }
            }
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "trim" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::Text(ref t) => t.clone(),
                x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "text")))
            };
            rt.stack.push(Variable::Text(Arc::new(v.trim().into())));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "trim_left" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::Text(ref t) => t.clone(),
                x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "text")))
            };
            rt.stack.push(Variable::Text(Arc::new(v.trim_left().into())));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "trim_right" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let mut v = match rt.resolve(&v) {
                &Variable::Text(ref t) => t.clone(),
                x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "text")))
            };
            {
                let w = Arc::make_mut(&mut v);
                while let Some(ch) = w.pop() {
                    if !ch.is_whitespace() { w.push(ch); break; }
                }
            }
            rt.stack.push(Variable::Text(v));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "to_string" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let mut buf: Vec<u8> = vec![];
            write_variable(&mut buf, rt, rt.resolve(&v), EscapeString::None).unwrap();
            let v = Variable::Text(Arc::new(String::from_utf8(buf).unwrap()));
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "to_string_color" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::Vec4(val) => val,
                x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "vec4")))
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
            rt.stack.push(Variable::Text(Arc::new(String::from_utf8(buf).unwrap())));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "srgb_to_linear_color" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::Vec4(val) => val,
                x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "vec4")))
            };
            let to_linear = |f: f32| {
                if f <= 0.04045 {
                    f / 12.92
                } else {
                    ((f + 0.055) / 1.055).powf(2.4)
                }
            };
            let v = [to_linear(v[0]), to_linear(v[1]), to_linear(v[2]), v[3]];
            rt.stack.push(Variable::Vec4(v));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "linear_to_srgb_color" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::Vec4(val) => val,
                x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "vec4")))
            };
            let to_srgb = |f: f32| {
                if f <= 0.0031308 {
                    f * 12.92
                } else {
                    1.055 * f.powf(1.0 / 2.4) - 0.055
                }
            };
            let v = [to_srgb(v[0]), to_srgb(v[1]), to_srgb(v[2]), v[3]];
            rt.stack.push(Variable::Vec4(v));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "typeof" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::Text(_) => rt.text_type.clone(),
                &Variable::F64(_) => rt.f64_type.clone(),
                &Variable::Vec4(_) => rt.vec4_type.clone(),
                &Variable::Return => rt.return_type.clone(),
                &Variable::Bool(_) => rt.bool_type.clone(),
                &Variable::Object(_) => rt.object_type.clone(),
                &Variable::Array(_) => rt.array_type.clone(),
                &Variable::Ref(_) => rt.ref_type.clone(),
                &Variable::UnsafeRef(_) => rt.unsafe_ref_type.clone(),
                &Variable::RustObject(_) => rt.rust_object_type.clone(),
                &Variable::Option(_) => rt.option_type.clone(),
                &Variable::Result(_) => rt.result_type.clone(),
                &Variable::Thread(_) => rt.thread_type.clone(),
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "debug" => {
            rt.push_fn(call.name.clone(), None, st, lc);
            println!("Stack {:#?}", rt.stack);
            println!("Locals {:#?}", rt.local_stack);
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "backtrace" => {
            rt.push_fn(call.name.clone(), None, st, lc);
            println!("{:#?}", rt.call_stack);
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "load" => {
            use load;

            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::Text(ref text) => {
                    let mut m = Module::new();
                    for (key, &(ref f, ref ext)) in &*module.ext_prelude {
                        m.add(key.clone(), *f, ext.clone());
                    }
                    if let Err(err) = load(text, &mut m) {
                        Variable::Result(Err(Box::new(Error {
                            message: Variable::Text(Arc::new(
                                format!("{}\n{}\n{}", rt.stack_trace(), err,
                                    module.error(call.args[0].source_range(),
                                    "When attempting to load module:")))),
                            trace: vec![]
                        })))
                    } else {
                        Variable::Result(Ok(Box::new(
                            Variable::RustObject(Arc::new(Mutex::new(m))))))
                    }
                }
                x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "string")))
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "load_source_imports" => {
            use load;

            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let modules = rt.stack.pop().expect(TINVOTS);
            let source = rt.stack.pop().expect(TINVOTS);
            let mut new_module = Module::new();
            for (key, &(ref f, ref ext)) in &*module.ext_prelude {
                new_module.add(key.clone(), *f, ext.clone());
            }
            match rt.resolve(&modules) {
                &Variable::Array(ref array) => {
                    for it in &**array {
                        match rt.resolve(it) {
                            &Variable::RustObject(ref obj) => {
                                match obj.lock().unwrap().downcast_ref::<Module>() {
                                    Some(m) => {
                                        for f in m.functions.values() {
                                            new_module.register(f.clone())
                                        }
                                    }
                                    None => return Err(module.error(
                                        call.args[1].source_range(),
                                        &format!("{}\nExpected `Module`",
                                            rt.stack_trace())))
                                }
                            }
                            x => return Err(module.error(
                                call.args[1].source_range(),
                                &rt.expected(x, "Module")))
                        }
                    }
                }
                x => return Err(module.error(call.args[1].source_range(),
                        &rt.expected(x, "[Module]")))
            }
            let v = match rt.resolve(&source) {
                &Variable::Text(ref text) => {
                    if let Err(err) = load(text, &mut new_module) {
                        Variable::Result(Err(Box::new(Error {
                            message: Variable::Text(Arc::new(
                                format!("{}\n{}\n{}", rt.stack_trace(), err,
                                    module.error(call.args[0].source_range(),
                                    "When attempting to load module:")))),
                            trace: vec![]
                        })))
                    } else {
                        Variable::Result(Ok(Box::new(
                            Variable::RustObject(Arc::new(
                                Mutex::new(new_module))))))
                    }
                }
                x => return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "[Module]")))
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "call" => {
            rt.push_fn(call.name.clone(), None, st, lc);
            let args = rt.stack.pop().expect(TINVOTS);
            let fn_name = rt.stack.pop().expect(TINVOTS);
            let call_module = rt.stack.pop().expect(TINVOTS);
            let fn_name = match rt.resolve(&fn_name) {
                &Variable::Text(ref text) => text.clone(),
                x => return Err(module.error(call.args[1].source_range(),
                                &rt.expected(x, "text")))
            };
            let args = match rt.resolve(&args) {
                &Variable::Array(ref arr) => arr.clone(),
                x => return Err(module.error(call.args[2].source_range(),
                                &rt.expected(x, "array")))
            };
            let obj = match rt.resolve(&call_module) {
                &Variable::RustObject(ref obj) => obj.clone(),
                x => return Err(module.error(call.args[0].source_range(),
                                &rt.expected(x, "Module")))
            };

            match obj.lock().unwrap()
                .downcast_ref::<Module>() {
                Some(m) => {
                    match m.functions.get(&fn_name) {
                        Some(ref f) => {
                            if f.args.len() != args.len() {
                                return Err(module.error(
                                    call.args[2].source_range(),
                                    &format!(
                                        "{}\nExpected `{}` arguments, found `{}`",
                                        rt.stack_trace(),
                                        f.args.len(), args.len())))
                            }
                        }
                        None => return Err(module.error(
                                    call.args[1].source_range(),
                                    &format!(
                                        "{}\nCould not find function `{}`",
                                        rt.stack_trace(),
                                        fn_name)))
                    }
                    let call = ast::Call {
                        name: fn_name.clone(),
                        args: args.iter().map(|arg|
                            ast::Expression::Variable(
                                call.source_range, arg.clone())).collect(),
                        source_range: call.source_range,
                    };
                    // TODO: Figure out what to do expect and flow.
                    try!(rt.call(&call, &m));
                }
                None => return Err(module.error(call.args[0].source_range(),
                            &format!("{}\nExpected `Module`",
                                rt.stack_trace())))
            }

            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "call_ret" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let args = rt.stack.pop().expect(TINVOTS);
            let fn_name = rt.stack.pop().expect(TINVOTS);
            let call_module = rt.stack.pop().expect(TINVOTS);
            let fn_name = match rt.resolve(&fn_name) {
                &Variable::Text(ref text) => text.clone(),
                x => return Err(module.error(call.args[1].source_range(),
                                &rt.expected(x, "text")))
            };
            let args = match rt.resolve(&args) {
                &Variable::Array(ref arr) => arr.clone(),
                x => return Err(module.error(call.args[2].source_range(),
                                &rt.expected(x, "array")))
            };
            let obj = match rt.resolve(&call_module) {
                &Variable::RustObject(ref obj) => obj.clone(),
                x => return Err(module.error(call.args[0].source_range(),
                                &rt.expected(x, "Module")))
            };

            match obj.lock().unwrap()
                .downcast_ref::<Module>() {
                Some(m) => {
                    match m.functions.get(&fn_name) {
                        Some(ref f) => {
                            if f.args.len() != args.len() {
                                return Err(module.error(
                                    call.args[2].source_range(),
                                    &format!(
                                        "{}\nExpected `{}` arguments, found `{}`",
                                        rt.stack_trace(),
                                        f.args.len(), args.len())))
                            }
                        }
                        None => return Err(module.error(
                                    call.args[1].source_range(),
                                    &format!(
                                        "{}\nCould not find function `{}`",
                                        rt.stack_trace(),
                                        fn_name)))
                    }
                    let call = ast::Call {
                        name: fn_name.clone(),
                        args: args.iter().map(|arg|
                            ast::Expression::Variable(
                                call.source_range, arg.clone())).collect(),
                        source_range: call.source_range,
                    };
                    // TODO: Figure out what to do expect and flow.
                    try!(rt.call(&call, &m));
                }
                None => return Err(module.error(call.args[0].source_range(),
                    &format!("{}\nExpected `Module`", rt.stack_trace())))
            }

            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "functions" => {
            // List available functions in scope.
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let mut functions = vec![];
            let name: Arc<String> = Arc::new("name".into());
            let arguments: Arc<String> = Arc::new("arguments".into());
            let returns: Arc<String> = Arc::new("returns".into());
            let takes: Arc<String> = Arc::new("takes".into());
            let lifetime: Arc<String> = Arc::new("lifetime".into());
            let ret_lifetime: Arc<String> = Arc::new("return".into());
            let ty: Arc<String> = Arc::new("type".into());
            let intrinsic: Arc<String> = Arc::new("intrinsic".into());
            let external: Arc<String> = Arc::new("external".into());
            let loaded: Arc<String> = Arc::new("loaded".into());
            let mut intrinsics = HashMap::new();
            standard(&mut intrinsics);
            for (f_name, f) in &intrinsics {
                let mut obj = HashMap::new();
                obj.insert(name.clone(), Variable::Text(f_name.clone()));
                obj.insert(returns.clone(), Variable::Text(Arc::new(f.ret.description())));
                obj.insert(ty.clone(), Variable::Text(intrinsic.clone()));
                let mut args = vec![];
                for (i, lt) in f.lts.iter().enumerate() {
                    let mut obj_arg = HashMap::new();
                    obj_arg.insert(name.clone(),
                        Variable::Text(Arc::new(format!("arg{}", i).into())));
                    obj_arg.insert(lifetime.clone(), match *lt {
                        Lt::Default => Variable::Option(None),
                        Lt::Arg(ind) => Variable::Option(Some(
                                Box::new(Variable::Text(
                                    Arc::new(format!("arg{}", ind).into())
                                ))
                            )),
                        Lt::Return => Variable::Option(Some(
                                Box::new(Variable::Text(ret_lifetime.clone()))
                            )),
                    });
                    obj_arg.insert(takes.clone(),
                        Variable::Text(Arc::new(f.tys[i].description())));
                    args.push(Variable::Object(Arc::new(obj_arg)));
                }
                obj.insert(arguments.clone(), Variable::Array(Arc::new(args)));
                functions.push(Variable::Object(Arc::new(obj)));
            }
            for (f_name, &(_, ref f)) in &*module.ext_prelude {
                let mut obj = HashMap::new();
                obj.insert(name.clone(), Variable::Text(f_name.clone()));
                obj.insert(returns.clone(), Variable::Text(Arc::new(f.ret.description())));
                obj.insert(ty.clone(), Variable::Text(external.clone()));
                let mut args = vec![];
                for (i, lt) in f.lts.iter().enumerate() {
                    let mut obj_arg = HashMap::new();
                    obj_arg.insert(name.clone(),
                        Variable::Text(Arc::new(format!("arg{}", i).into())));
                    obj_arg.insert(lifetime.clone(), match *lt {
                        Lt::Default => Variable::Option(None),
                        Lt::Arg(ind) => Variable::Option(Some(
                                Box::new(Variable::Text(
                                    Arc::new(format!("arg{}", ind).into())
                                ))
                            )),
                        Lt::Return => Variable::Option(Some(
                                Box::new(Variable::Text(ret_lifetime.clone()))
                            )),
                    });
                    obj_arg.insert(takes.clone(),
                        Variable::Text(Arc::new(f.tys[i].description())));
                    args.push(Variable::Object(Arc::new(obj_arg)));
                }
                obj.insert(arguments.clone(), Variable::Array(Arc::new(args)));
                functions.push(Variable::Object(Arc::new(obj)));
            }
            for f in module.functions.values() {
                let mut obj = HashMap::new();
                obj.insert(name.clone(), Variable::Text(f.name.clone()));
                obj.insert(returns.clone(), Variable::Text(Arc::new(f.ret.description())));
                obj.insert(ty.clone(), Variable::Text(loaded.clone()));
                let mut args = vec![];
                for arg in &f.args {
                    let mut obj_arg = HashMap::new();
                    obj_arg.insert(name.clone(),
                        Variable::Text(arg.name.clone()));
                    obj_arg.insert(lifetime.clone(),
                        match arg.lifetime {
                            None => Variable::Option(None),
                            Some(ref lt) => Variable::Option(Some(Box::new(
                                    Variable::Text(lt.clone())
                                )))
                        }
                    );
                    obj_arg.insert(takes.clone(),
                        Variable::Text(Arc::new(arg.ty.description())));
                    args.push(Variable::Object(Arc::new(obj_arg)));
                }
                obj.insert(arguments.clone(), Variable::Array(Arc::new(args)));
                functions.push(Variable::Object(Arc::new(obj)));
            }
            // Sort by function names.
            functions.sort_by(|a, b|
                match (a, b) {
                    (&Variable::Object(ref a), &Variable::Object(ref b)) => {
                        match (&a[&name], &b[&name]) {
                            (&Variable::Text(ref a), &Variable::Text(ref b)) => {
                                a.cmp(b)
                            }
                            _ => panic!("Expected two strings")
                        }
                    }
                    _ => panic!("Expected two objects")
                }
            );
            let v = Variable::Array(Arc::new(functions));
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "none" => {
            rt.stack.push(Variable::Option(None));
            Expect::Something
        }
        "some" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = rt.resolve(&v).deep_clone(&rt.stack);
            rt.stack.push(Variable::Option(Some(Box::new(v))));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "ok" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = rt.resolve(&v).deep_clone(&rt.stack);
            rt.stack.push(Variable::Result(Ok(Box::new(v))));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "err" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = rt.resolve(&v).deep_clone(&rt.stack);
            rt.stack.push(Variable::Result(Err(Box::new(
                Error { message: v, trace: vec![] }))));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "is_err" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::Result(Err(_)) => Variable::Bool(true),
                &Variable::Result(Ok(_)) => Variable::Bool(false),
                x => {
                    return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "option")));
                }
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "unwrap" => {
            // Return value does not depend on lifetime of argument since
            // `ok(x)` and `some(x)` perform a deep clone.
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::Option(Some(ref v)) => (**v).clone(),
                &Variable::Option(None) => {
                    return Err(module.error(call.args[0].source_range(),
                        &format!("{}\nExpected `some(_)`",
                            rt.stack_trace())));
                }
                &Variable::Result(Ok(ref ok)) => (**ok).clone(),
                &Variable::Result(Err(ref err)) => {
                    use std::str::from_utf8;

                    // Print out error message.
                    let mut w: Vec<u8> = vec![];
                    w.extend_from_slice(rt.stack_trace().as_bytes());
                    w.extend_from_slice("\n".as_bytes());
                    write_variable(&mut w, rt, &err.message,
                                   EscapeString::None).unwrap();
                    for t in &err.trace {
                        w.extend_from_slice("\n".as_bytes());
                        w.extend_from_slice(t.as_bytes());
                    }
                    return Err(module.error(call.args[0].source_range(),
                                            from_utf8(&w).unwrap()));
                }
                x => {
                    return Err(module.error(call.args[0].source_range(),
                                            &rt.expected(x, "some(_) or ok(_)")));
                }
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "unwrap_err" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::Result(Err(ref err)) => err.message.clone(),
                x => {
                    return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "err(_)")));
                }
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "dir_angle" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::F64(val) => Variable::Vec4([val.cos() as f32, val.sin() as f32, 0.0, 0.0]),
                x => {
                    return Err(module.error(call.args[0].source_range(),
                        &rt.expected(x, "err(_)")));
                }
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "load_meta_file" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let file = rt.stack.pop().expect(TINVOTS);
            let meta = rt.stack.pop().expect(TINVOTS);
            let file = match rt.resolve(&file) {
                &Variable::Text(ref file) => file.clone(),
                x => return Err(module.error(call.args[1].source_range(),
                                &rt.expected(x, "str")))
            };
            let meta = match rt.resolve(&meta) {
                &Variable::Text(ref meta) => meta.clone(),
                x => return Err(module.error(call.args[0].source_range(),
                                &rt.expected(x, "str")))
            };
            let res = meta::load_meta_file(&**meta, &**file);
            rt.stack.push(Variable::Result(match res {
                Ok(res) => Ok(Box::new(Variable::Array(Arc::new(res)))),
                Err(err) => Err(Box::new(Error {
                    message: Variable::Text(Arc::new(err)),
                    trace: vec![]
                }))
            }));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "load_meta_url" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let url = rt.stack.pop().expect(TINVOTS);
            let meta = rt.stack.pop().expect(TINVOTS);
            let url = match rt.resolve(&url) {
                &Variable::Text(ref url) => url.clone(),
                x => return Err(module.error(call.args[1].source_range(),
                                &rt.expected(x, "str")))
            };
            let meta = match rt.resolve(&meta) {
                &Variable::Text(ref meta) => meta.clone(),
                x => return Err(module.error(call.args[0].source_range(),
                                &rt.expected(x, "str")))
            };
            let res = meta::load_meta_url(&**meta, &**url);
            rt.stack.push(Variable::Result(match res {
                Ok(res) => Ok(Box::new(Variable::Array(Arc::new(res)))),
                Err(err) => Err(Box::new(Error {
                    message: Variable::Text(Arc::new(err)),
                    trace: vec![]
                }))
            }));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "join_thread" => {
            use Thread;

            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let thread = rt.stack.pop().expect(TINVOTS);
            let handle_res = Thread::invalidate_handle(rt, thread);
            rt.stack.push(Variable::Result({
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
            }));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        _ => return Err(module.error(call.source_range,
            &format!("{}\nUnknown function `{}`", rt.stack_trace(), call.name)))
    };
    Ok((expect, Flow::Continue))
}
