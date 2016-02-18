use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rand::Rng;

use runtime::{Expect, Flow, Runtime, Side};
use ast;

use Variable;
use Module;

pub fn standard() -> HashMap<&'static str, Intrinsic> {
    let mut i: HashMap<&'static str, Intrinsic> = HashMap::new();
    i.insert("println", PRINTLN);
    i.insert("print", PRINT);
    i.insert("clone", CLONE);
    i.insert("debug", DEBUG);
    i.insert("backtrace", BACKTRACE);
    i.insert("sleep", SLEEP);
    i.insert("round", ROUND);
    i.insert("random", RANDOM);
    i.insert("read_number", READ_NUMBER);
    i.insert("read_line", READ_LINE);
    i.insert("len", LEN);
    i.insert("push", PUSH);
    i.insert("trim_right", TRIM_RIGHT);
    i.insert("to_string", TO_STRING);
    i.insert("typeof", TYPEOF);
    i.insert("sqrt", SQRT);
    i.insert("sin", SIN);
    i.insert("asin", ASIN);
    i.insert("cos", COS);
    i.insert("acos", ACOS);
    i.insert("tan", TAN);
    i.insert("atan", ATAN);
    i.insert("exp", EXP);
    i.insert("ln", LN);
    i.insert("log2", LOG2);
    i.insert("log10", LOG10);
    i.insert("random", RANDOM);
    i.insert("load", LOAD);
    i.insert("load_source_imports", LOAD_SOURCE_IMPORTS);
    i.insert("call", CALL);
    i
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
        RustObject(_) => v.clone()
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
        _ => panic!("Unknown function `{}`", call.name)
    };
    Ok((expect, Flow::Continue))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArgConstraint {
    Arg(usize),
    Return,
    Default,
}

#[derive(Debug, Copy, Clone)]
pub struct Intrinsic {
    pub arg_constraints: &'static [ArgConstraint],
    pub returns: bool,
}

static PRINTLN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static PRINT: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static CLONE: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static DEBUG: Intrinsic = Intrinsic {
    arg_constraints: &[],
    returns: false
};

static BACKTRACE: Intrinsic = Intrinsic {
    arg_constraints: &[],
    returns: false
};

static SLEEP: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static ROUND: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static RANDOM: Intrinsic = Intrinsic {
    arg_constraints: &[],
    returns: true
};

static READ_NUMBER: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static READ_LINE: Intrinsic = Intrinsic {
    arg_constraints: &[],
    returns: true
};

static TRIM_RIGHT: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LEN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static PUSH: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default, ArgConstraint::Arg(0)],
    returns: false
};

static SQRT: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static ASIN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static SIN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static COS: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static ACOS: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TAN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static ATAN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static EXP: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOG2: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOG10: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TO_STRING: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TYPEOF: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOAD: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOAD_SOURCE_IMPORTS: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default; 2],
    returns: true
};

static CALL: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default; 3],
    returns: true
};
