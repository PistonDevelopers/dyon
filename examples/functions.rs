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
    use std::sync::Arc;
    use dyon::*;

    let mut module = Module::new();
    module.add(Arc::new("say_hello".into()), say_hello, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::Void
    });
    module.add(Arc::new("homer".into()), homer, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::Any
    });
    module.add(Arc::new("age".into()), age, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::Any],
        ret: Type::Any
    });
    module.add(Arc::new("mr".into()), mr, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Text; 2],
        ret: Type::Text
    });
    module.add(Arc::new("origo".into()), origo, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::Object,
    });
    module.add(Arc::new("id".into()), id, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::Mat4,
    });

    // Register custom Rust object with an ad-hoc type.
    let ty_custom_object = Type::AdHoc(Arc::new("CustomObject".into()), Box::new(Type::Any));
    module.add(Arc::new("custom_object".into()), custom_object, Dfn {
        lts: vec![],
        tys: vec![],
        ret: ty_custom_object.clone(),
    });
    module.add(Arc::new("print_custom_object".into()), print_custom_object, Dfn {
        lts: vec![Lt::Default],
        tys: vec![ty_custom_object.clone()],
        ret: Type::Void,
    });
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
dyon_fn!{fn custom_object() -> RustObject {
    use std::sync::{Arc, Mutex};

    let val: i32 = 42;
    Arc::new(Mutex::new(val)) as RustObject
}}

// Print out the content of a custom Rust object.
dyon_fn!{fn print_custom_object(obj: RustObject) {
    let a_guard = obj.lock().unwrap();
    let a = a_guard.downcast_ref::<i32>().unwrap();
    println!("Custom value is {}", a);
}}

dyon_fn!{fn id() -> Mat4 {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0]
    ].into()
}}
