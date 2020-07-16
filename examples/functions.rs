#[macro_use]
extern crate dyon;

use std::sync::Arc;
use dyon::{RustObject, Vec4, Mat4};

fn main() {
    use dyon::{error, Runtime};

    let mut dyon_runtime = Runtime::new();
    let dyon_module = load_module().unwrap();
    if error(dyon_runtime.run(&Arc::new(dyon_module))) {
        return
    }
}

fn load_module() -> Option<dyon::Module> {
    use dyon::{error, load, Dfn, Module};
    use dyon::Type::*;

    let mut module = Module::new();
    module.add_str("say_hello", say_hello, Dfn::nl(vec![], Void));
    module.add_str("homer", homer, Dfn::nl(vec![], Any));
    module.add_str("age", age, Dfn::nl(vec![Any], Any));
    module.add_str("mr", mr, Dfn::nl(vec![Str; 2], Str));
    module.add_str("origo", origo, Dfn::nl(vec![], Object));
    module.add_str("id", id, Dfn::nl(vec![], Mat4));

    // Register custom Rust object with an ad-hoc type.
    let ty_custom_object = AdHoc(Arc::new("CustomObject".into()), Box::new(Any));
    module.add_str("custom_object", custom_object, Dfn::nl(vec![], ty_custom_object.clone()));
    module.add_str("print_custom_object", print_custom_object,
        Dfn::nl(vec![ty_custom_object.clone()], Void));
    if error(load("source/functions/loader.dyon", &mut module)) {
        None
    } else {
        Some(module)
    }
}

dyon_fn!{fn say_hello() {
    println!("hi!");
}}

dyon_fn!{fn homer() -> Person {
    Person {
        first_name: "Homer".into(),
        last_name: "Simpson".into(),
        age: 48
    }
}}

dyon_fn!{fn age(person: Person) -> Person {
    Person { age: person.age + 1, ..person }
}}

dyon_fn!{fn mr(first_name: String, last_name: String) -> String {
    format!("Mr {} {}", first_name, last_name)
}}

pub struct Person {
    pub first_name: String,
    pub last_name: String,
    pub age: u32,
}

dyon_obj!{Person { first_name, last_name, age }}

pub struct PhysicalState {
    pub pos: Vec4,
    pub vel: Vec4
}

dyon_obj!{PhysicalState { pos, vel }}

dyon_fn!{fn origo() -> PhysicalState {
    PhysicalState {
        pos: [0.0, 1.0, 2.0].into(),
        vel: [3.0, 4.0, 5.0].into()
    }
}}

// Create a custom Rust object.
dyon_fn!{fn custom_object() -> #i32 {42}}

// Print out the content of a custom Rust object.
dyon_fn!{fn print_custom_object(a: #i32) {
    println!("Custom value is {}", a);
}}

// Macro example.
dyon_fn!{fn foo1(a: #i32, _b: f64) {
    println!("Custom value is {}", a);
}}

// Macro example.
dyon_fn!{fn foo2(a: #i32, b: f64) -> f64 {
    a as f64 + b
}}

// Macro example.
dyon_fn!{fn foo3(a: #&i32, b: f64) -> f64 {
    *a as f64 + b
}}

// Macro example.
dyon_fn!{fn foo4(a: #&i32, b: #&i32) -> #i32 {
    a + b
}}

// Macro example.
dyon_fn!{fn foo5(a: #&i32, b: f64) -> #f64 {
    *a as f64 + b
}}

// Macro example.
dyon_fn!{fn foo6(a: f64) -> #i32 {
    a as i32
}}

// Macro example.
dyon_fn!{fn foo7(a: #&i32) -> f64 {
    *a as f64
}}

// Macro example.
dyon_fn!{fn foo8(a: #&mut f64, b: f64) {
    *a = b
}}

// Macro example.
dyon_fn!{fn foo9(a: #&mut f64, b: #f64) {
    *a = b
}}

// Macro example.
dyon_fn!{fn foo10(a: #&mut f64, b: #f64) -> f64 {
    *a = b;
    *a
}}

dyon_fn!{fn id() -> Mat4 {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0]
    ].into()
}}
