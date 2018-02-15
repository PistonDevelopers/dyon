#[macro_use]
extern crate dyon;

use std::sync::Arc;
use dyon::{load_str, error, Call, Module, Dfn, Lt, Type, RustObject};

fn main() {
    let mut module = Module::new();

    // Add functions to read `a` and `b` from `RustArgs`.
    module.add(Arc::new("a_of".into()), a_of, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::Any],
        ret: Type::F64
    });
    module.add(Arc::new("b_of".into()), b_of, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::Any],
        ret: Type::F64
    });

    error(load_str("main.dyon", Arc::new(r#"
        fn add_args(a: f64, b: f64) {
            println("add_args:")
            println(link {a" + "b" = "a + b})
        }

        fn add_obj(obj: {}) {
            println("add_obj:")
            println(link {obj.a" + "obj.b" = "obj.a + obj.b})
        }

        fn add_rust(obj: any) {
            println("add_rust")
            a := a_of(obj)
            b := b_of(obj)
            println(link {a" + "b" = "a + b})
        }

        add(a, b) = a + b

        create_vec(a, b) = (a, b)

        id(obj) = clone(obj)
    "#.into()), &mut module));
    let ref module = Arc::new(module);

    let a = 20.0;
    let b = 30.0;

    // Call with multiple arguments.
    let call = Call::new("add_args").arg(a).arg(b);
    error(call.run(module));

    // Call with object.
    let call = Call::new("add_obj").arg(Args {a, b});
    error(call.run(module));

    // Call with rust object.
    let call = Call::new("add_rust").rust(RustArgs {a, b});
    error(call.run(module));

    // Call function with return value.
    let call = Call::new("add").arg(a).arg(b);
    match call.run_ret::<f64>(module) {
        Ok(answer) => {println!("{}", answer);}
        Err(err) => {error(Err(err));}
    }

    // Call function that returns vec4.
    let call = Call::new("create_vec").arg(a).arg(b);
    match call.run_vec4::<[f64; 2]>(module) {
        Ok(answer) => {println!("{:?}", answer);}
        Err(err) => {error(Err(err));}
    }

    // Call function that returns Rust object.
    let call = Call::new("id").rust(RustArgs {a, b});
    match call.run_ret::<RustObject>(module) {
        Ok(answer) => {println!("{:?}", answer.lock().unwrap().downcast_ref::<RustArgs>());}
        Err(err) => {error(Err(err));}
    }
}

struct Args {
    a: f64,
    b: f64,
}

dyon_obj!{Args {a, b}}

#[derive(Debug)]
struct RustArgs {
    a: f64,
    b: f64,
}

dyon_fn!{fn a_of(obj: RustObject) -> f64 {
    let obj_guard = obj.lock().unwrap();
    obj_guard.downcast_ref::<RustArgs>().unwrap().a
}}

dyon_fn!{fn b_of(obj: RustObject) -> f64 {
    let obj_guard = obj.lock().unwrap();
    obj_guard.downcast_ref::<RustArgs>().unwrap().b
}}
