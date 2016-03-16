use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rand::Rng;

use runtime::{Expect, Flow, Runtime, Side};
use ast;
use prelude::{ArgConstraint, PreludeFunction};

use Variable;
use Module;

pub fn standard(f: &mut HashMap<Arc<String>, PreludeFunction>) {
    f.insert(Arc::new("println".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: false
    });
    f.insert(Arc::new("print".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: false
    });
    f.insert(Arc::new("clone".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: false
    });
    f.insert(Arc::new("debug".into()), PreludeFunction {
        arg_constraints: vec![],
        returns: false
    });
    f.insert(Arc::new("backtrace".into()), PreludeFunction {
        arg_constraints: vec![],
        returns: false
    });
    f.insert(Arc::new("sleep".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: false
    });
    f.insert(Arc::new("round".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("random".into()), PreludeFunction {
        arg_constraints: vec![],
        returns: true
    });
    f.insert(Arc::new("read_number".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("read_line".into()), PreludeFunction {
        arg_constraints: vec![],
        returns: true
    });
    f.insert(Arc::new("len".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("push".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default, ArgConstraint::Arg(0)],
        returns: false
    });
    f.insert(Arc::new("trim_right".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("to_string".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("typeof".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("sqrt".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("sin".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("asin".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("cos".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("acos".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("tan".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("atan".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("exp".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("ln".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("log2".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("log10".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("load".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default],
        returns: true
    });
    f.insert(Arc::new("load_source_imports".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default; 2],
        returns: true
    });
    f.insert(Arc::new("call".into()), PreludeFunction {
        arg_constraints: vec![ArgConstraint::Default; 3],
        returns: true
    });
    f.insert(Arc::new("functions".into()), PreludeFunction {
        arg_constraints: vec![],
        returns: true
    });
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
            for (_, val) in &mut res {
                *val = deep_clone(val, stack);
            }
            Object(res)
        }
        Array(ref arr) => {
            let mut res = arr.clone();
            for it in &mut res {
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
        Option(Some(ref v)) => deep_clone(v, stack)
    }
}

fn print_variable(rt: &Runtime, v: &Variable) {
    match *rt.resolve(v) {
        Variable::Text(ref t) => {
            print!("{}", t);
        }
        Variable::F64(x) => {
            print!("{}", x);
        }
        Variable::Bool(x) => {
            print!("{}", x);
        }
        Variable::Ref(ind) => {
            print_variable(rt, &rt.stack[ind]);
        }
        Variable::Object(ref obj) => {
            print!("{{");
            let n = obj.len();
            for (i, (k, v)) in obj.iter().enumerate() {
                print!("{}: ", k);
                print_variable(rt, v);
                if i + 1 < n {
                    print!(", ");
                }
            }
            print!("}}");
        }
        Variable::Array(ref arr) => {
            print!("[");
            let n = arr.len();
            for (i, v) in arr.iter().enumerate() {
                print_variable(rt, v);
                if i + 1 < n {
                    print!(", ");
                }
            }
            print!("]");
        }
        ref x => panic!("Could not print out `{:?}`", x)
    }
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
                            "Expected something. \
                            Expression did not return a value."))
        };
    }
    let expect = match &**call.name {
        "clone" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = rt.stack.pop()
                .expect("There is no value on the stack");
            let v = deep_clone(rt.resolve(&v), &rt.stack);
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "println" => {
            rt.push_fn(call.name.clone(), st, lc);
            let x = rt.stack.pop()
                .expect("There is no value on the stack");
            print_variable(rt, &x);
            println!("");
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "print" => {
            rt.push_fn(call.name.clone(), st, lc);
            let x = rt.stack.pop()
                .expect("There is no value on the stack");
            print_variable(rt, &x);
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
        "sleep" => {
            use std::thread::sleep;
            use std::time::Duration;

            rt.push_fn(call.name.clone(), st, lc);
            let v = match rt.stack.pop() {
                Some(Variable::F64(b)) => b,
                Some(_) => return Err(module.error(call.args[0].source_range(),
                                      "Expected number")),
                None => panic!("There is no value on the stack")
            };
            let secs = v as u64;
            let nanos = (v.fract() * 1.0e9) as u32;
            sleep(Duration::new(secs, nanos));
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "random" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = Variable::F64(rt.rng.gen());
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "round" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = match rt.stack.pop() {
                Some(Variable::F64(b)) => b,
                Some(_) => return Err(module.error(call.args[0].source_range(),
                                      "Expected number")),
                None => panic!("There is no value on the stack")
            };
            let v = Variable::F64(v.round());
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "len" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = match rt.stack.pop() {
                Some(v) => v,
                None => panic!("There is no value on the stack")
            };

            let v = {
                let arr = match rt.resolve(&v) {
                    &Variable::Array(ref arr) => arr,
                    _ => return Err(module.error(call.args[0].source_range(),
                                    "Expected array"))
                };
                Variable::F64(arr.len() as f64)
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "push" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let item = match rt.stack.pop() {
                Some(item) => item,
                None => panic!("There is no value on the stack")
            };
            let v = match rt.stack.pop() {
                Some(v) => v,
                None => panic!("There is no value on the stack")
            };

            if let Variable::Ref(ind) = v {
                if let Variable::Array(ref mut arr) =
                rt.stack[ind] {
                    arr.push(item);
                } else {
                    return Err(module.error(call.args[0].source_range(),
                               "Expected reference to array"));
                }
            } else {
                return Err(module.error(call.args[0].source_range(),
                           "Expected reference to array"));
            }
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "read_line" => {
            use std::io::{self, Write};

            rt.push_fn(call.name.clone(), st + 1, lc);
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

            rt.push_fn(call.name.clone(), st + 1, lc);
            let err = rt.stack.pop()
                    .expect("There is no value on the stack");
            let err = match rt.resolve(&err) {
                &Variable::Text(ref t) => t.clone(),
                _ => return Err(module.error(call.args[0].source_range(),
                                "Expected text"))
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
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = rt.stack.pop().expect("There is no value on the stack");
            let mut v = match rt.resolve(&v) {
                &Variable::Text(ref t) => t.clone(),
                _ => return Err(module.error(call.args[0].source_range(),
                                "Expected text"))
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
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = rt.stack.pop().expect("There is no value on the stack");
            let v = match rt.resolve(&v) {
                &Variable::Text(ref t) => Variable::Text(t.clone()),
                &Variable::F64(v) => {
                    Variable::Text(Arc::new(format!("{}", v)))
                }
                _ => unimplemented!(),
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "typeof" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = rt.stack.pop().expect("There is no value on the stack");
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
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "debug" => {
            rt.push_fn(call.name.clone(), st, lc);
            println!("Stack {:#?}", rt.stack);
            println!("Locals {:#?}", rt.local_stack);
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "backtrace" => {
            rt.push_fn(call.name.clone(), st, lc);
            println!("{:#?}", rt.call_stack);
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "load" => {
            use load;

            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = rt.stack.pop().expect("There is no value on the stack");
            let v = match rt.resolve(&v) {
                &Variable::Text(ref text) => {
                    let mut m = Module::new();
                    for (key, &(ref f, ref ext)) in &module.ext_prelude {
                        m.add(key.clone(), *f, ext.clone());
                    }
                    try!(load(text, &mut m).map_err(|err| {
                            format!("{}\n{}", err,
                                module.error(call.args[0].source_range(),
                                "When attempting to load module:"))
                        }));
                    Variable::RustObject(Arc::new(Mutex::new(m)))
                }
                _ => return Err(module.error(call.args[0].source_range(),
                                "Expected string"))
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "load_source_imports" => {
            use load;

            rt.push_fn(call.name.clone(), st + 1, lc);
            let modules = rt.stack.pop().expect("There is no value on the stack");
            let source = rt.stack.pop().expect("There is no value on the stack");
            let mut new_module = Module::new();
            for (key, &(ref f, ref ext)) in &module.ext_prelude {
                new_module.add(key.clone(), *f, ext.clone());
            }
            match rt.resolve(&modules) {
                &Variable::Array(ref array) => {
                    for it in array {
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
                                        "Expected `Module`"))
                                }
                            }
                            _ => return Err(module.error(
                                call.args[1].source_range(),
                                "Expected `Module`"))
                        }
                    }
                }
                _ => return Err(module.error(call.args[1].source_range(),
                    "Expected array of `Module`"))
            }
            let v = match rt.resolve(&source) {
                &Variable::Text(ref text) => {
                    try!(load(text, &mut new_module).map_err(|err| {
                            format!("{}\n{}", err,
                                module.error(call.args[0].source_range(),
                                "When attempting to load module:"))
                        }));
                    Variable::RustObject(
                        Arc::new(Mutex::new(new_module)))
                }
                _ => return Err(module.error(call.args[0].source_range(),
                    "Expected array of `Module`"))
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "call" => {
            rt.push_fn(call.name.clone(), st, lc);
            let args = rt.stack.pop().expect("There is no value on the stack");
            let fn_name = rt.stack.pop().expect("There is no value on the stack");
            let call_module = rt.stack.pop().expect("There is no value on the stack");
            let fn_name = match rt.resolve(&fn_name) {
                &Variable::Text(ref text) => text.clone(),
                _ => return Err(module.error(call.args[1].source_range(),
                                "Expected text"))
            };
            let args = match rt.resolve(&args) {
                &Variable::Array(ref arr) => arr.clone(),
                _ => return Err(module.error(call.args[2].source_range(),
                                "Expected array"))
            };
            let obj = match rt.resolve(&call_module) {
                &Variable::RustObject(ref obj) => obj.clone(),
                _ => return Err(module.error(call.args[0].source_range(),
                                "Expected `Module`"))
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
                                        "Expected `{}` arguments, found `{}`",
                                        f.args.len(), args.len())))
                            }
                        }
                        None => return Err(module.error(
                                    call.args[1].source_range(),
                                    &format!(
                                        "Could not find function `{}`",
                                        fn_name)))
                    }
                    let call = ast::Call {
                        name: fn_name.clone(),
                        args: args.into_iter().map(|arg|
                            ast::Expression::Variable(
                                call.source_range, arg)).collect(),
                        source_range: call.source_range,
                    };
                    // TODO: Figure out what to do expect and flow.
                    try!(rt.call(&call, &m));
                }
                None => return Err(module.error(call.args[0].source_range(),
                            "Expected `Module`"))
            }

            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "functions" => {
            // List available functions in scope.
            rt.push_fn(call.name.clone(), st + 1, lc);
            let mut functions = vec![];
            let name: Arc<String> = Arc::new("name".into());
            let arguments: Arc<String> = Arc::new("arguments".into());
            let returns: Arc<String> = Arc::new("returns".into());
            for f in module.functions.values() {
                let mut obj = HashMap::new();
                obj.insert(name.clone(), Variable::Text(f.name.clone()));
                obj.insert(returns.clone(), Variable::Bool(f.returns));
                let mut args = vec![];
                for arg in &f.args {
                    let mut obj_arg = HashMap::new();
                    obj_arg.insert(name.clone(),
                        Variable::Text(arg.name.clone()));
                    args.push(Variable::Object(obj_arg));
                }
                obj.insert(arguments.clone(), Variable::Array(args));
                functions.push(Variable::Object(obj));
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
            let v = Variable::Array(functions);
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        _ => panic!("Unknown function `{}`", call.name)
    };
    Ok((expect, Flow::Continue))
}
