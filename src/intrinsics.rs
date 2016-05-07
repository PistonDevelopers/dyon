use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io;
use rand::Rng;
use piston_meta::json;

use runtime::{Expect, Flow, Runtime, Side};
use ast;
use prelude::{Lt, PreludeFunction};

use Variable;
use Module;
use Error;

const TINVOTS: &'static str = "There is no value on the stack";

pub fn standard(f: &mut HashMap<Arc<String>, PreludeFunction>) {
    let sarg = |f: &mut HashMap<Arc<String>, PreludeFunction>, name: &str| {
        f.insert(Arc::new(name.into()), PreludeFunction {
            lts: vec![Lt::Default],
            returns: true
        });
    };

    sarg(f, "println");
    sarg(f, "print");
    sarg(f, "clone");
    f.insert(Arc::new("debug".into()), PreludeFunction {
        lts: vec![],
        returns: false
    });
    f.insert(Arc::new("backtrace".into()), PreludeFunction {
        lts: vec![],
        returns: false
    });
    sarg(f, "sleep");
    f.insert(Arc::new("random".into()), PreludeFunction {
        lts: vec![],
        returns: true
    });
    sarg(f, "read_number");
    f.insert(Arc::new("read_line".into()), PreludeFunction {
        lts: vec![],
        returns: true
    });
    sarg(f, "len");
    f.insert(Arc::new("push_ref(mut,_)".into()), PreludeFunction {
        lts: vec![Lt::Default, Lt::Arg(0)],
        returns: false
    });
    f.insert(Arc::new("push(mut,_)".into()), PreludeFunction {
        lts: vec![Lt::Default; 2],
        returns: false
    });
    f.insert(Arc::new("pop(mut)".into()), PreludeFunction {
        lts: vec![Lt::Return],
        returns: true
    });
    sarg(f, "trim_right");
    sarg(f, "to_string");
    sarg(f, "typeof");
    sarg(f, "round");
    sarg(f, "abs");
    sarg(f, "floor");
    sarg(f, "ceil");
    sarg(f, "sqrt");
    sarg(f, "sin");
    sarg(f, "asin");
    sarg(f, "cos");
    sarg(f, "acos");
    sarg(f, "tan");
    sarg(f, "atan");
    sarg(f, "exp");
    sarg(f, "ln");
    sarg(f, "log2");
    sarg(f, "log10");
    sarg(f, "load");
    f.insert(Arc::new("load_source_imports".into()), PreludeFunction {
        lts: vec![Lt::Default; 2],
        returns: true
    });
    f.insert(Arc::new("call".into()), PreludeFunction {
        lts: vec![Lt::Default; 3],
        returns: false
    });
    f.insert(Arc::new("call_ret".into()), PreludeFunction {
        lts: vec![Lt::Default; 3],
        returns: true
    });
    f.insert(Arc::new("functions".into()), PreludeFunction {
        lts: vec![],
        returns: true
    });
    f.insert(Arc::new("none".into()), PreludeFunction {
        lts: vec![],
        returns: true
    });
    sarg(f, "unwrap");
    sarg(f, "unwrap_err");
    sarg(f, "some");
    sarg(f, "ok");
    sarg(f, "err");
    sarg(f, "is_err");
}

fn deep_clone(v: &Variable, stack: &Vec<Variable>) -> Variable {
    use Variable::*;

    match *v {
        F64(_) => v.clone(),
        Return => v.clone(),
        Bool(_) => v.clone(),
        Text(_) => v.clone(),
        Object(ref obj) => {
            let mut res = obj.clone();
            for (_, val) in Arc::make_mut(&mut res) {
                *val = deep_clone(val, stack);
            }
            Object(res)
        }
        Array(ref arr) => {
            let mut res = arr.clone();
            for it in Arc::make_mut(&mut res) {
                *it = deep_clone(it, stack);
            }
            Array(res)
        }
        Ref(ind) => {
            deep_clone(&stack[ind], stack)
        }
        UnsafeRef(_) => panic!("Unsafe reference can not be cloned"),
        RustObject(_) => v.clone(),
        Option(None) => Variable::Option(None),
        // `some(x)` always uses deep clone, so it does not contain references.
        Option(Some(ref v)) => Option(Some(v.clone())),
        // `ok(x)` always uses deep clone, so it does not contain references.
        Result(Ok(ref ok)) => Result(Ok(ok.clone())),
        // `err(x)` always uses deep clone, so it does not contain references.
        Result(Err(ref err)) => Result(Err(err.clone())),
    }
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
        ref x => panic!("Could not print out `{:?}`", x)
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
    let expect = match &**call.name {
        "clone" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = deep_clone(rt.resolve(&v), &rt.stack);
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
            let item = deep_clone(rt.resolve(&item), &rt.stack);
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
        "typeof" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = match rt.resolve(&v) {
                &Variable::Text(_) => rt.text_type.clone(),
                &Variable::F64(_) => rt.f64_type.clone(),
                &Variable::Return => rt.return_type.clone(),
                &Variable::Bool(_) => rt.bool_type.clone(),
                &Variable::Object(_) => rt.object_type.clone(),
                &Variable::Array(_) => rt.array_type.clone(),
                &Variable::Ref(_) => rt.ref_type.clone(),
                &Variable::UnsafeRef(_) => rt.unsafe_ref_type.clone(),
                &Variable::RustObject(_) => rt.rust_object_type.clone(),
                &Variable::Option(_) => rt.option_type.clone(),
                &Variable::Result(_) => rt.result_type.clone(),
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
                    for (key, &(ref f, ref ext)) in &module.ext_prelude {
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
            for (key, &(ref f, ref ext)) in &module.ext_prelude {
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
                obj.insert(returns.clone(), Variable::Bool(f.returns));
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
                    args.push(Variable::Object(Arc::new(obj_arg)));
                }
                obj.insert(arguments.clone(), Variable::Array(Arc::new(args)));
                functions.push(Variable::Object(Arc::new(obj)));
            }
            for (f_name, &(_, ref f)) in &module.ext_prelude {
                let mut obj = HashMap::new();
                obj.insert(name.clone(), Variable::Text(f_name.clone()));
                obj.insert(returns.clone(), Variable::Bool(f.returns));
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
                    args.push(Variable::Object(Arc::new(obj_arg)));
                }
                obj.insert(arguments.clone(), Variable::Array(Arc::new(args)));
                functions.push(Variable::Object(Arc::new(obj)));
            }
            for f in module.functions.values() {
                let mut obj = HashMap::new();
                obj.insert(name.clone(), Variable::Text(f.name.clone()));
                obj.insert(returns.clone(), Variable::Bool(f.returns));
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
            let v = deep_clone(rt.resolve(&v), &rt.stack);
            rt.stack.push(Variable::Option(Some(Box::new(v))));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "ok" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = deep_clone(rt.resolve(&v), &rt.stack);
            rt.stack.push(Variable::Result(Ok(Box::new(v))));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "err" => {
            rt.push_fn(call.name.clone(), None, st + 1, lc);
            let v = rt.stack.pop().expect(TINVOTS);
            let v = deep_clone(rt.resolve(&v), &rt.stack);
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
        _ => return Err(module.error(call.source_range,
            &format!("{}\nUnknown function `{}`", rt.stack_trace(), call.name)))
    };
    Ok((expect, Flow::Continue))
}
