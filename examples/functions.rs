extern crate dyon;

use std::sync::Arc;
use std::collections::HashMap;
use dyon::*;

fn main() {
    let mut dyon_runtime = Runtime::new();
    let dyon_module = load_module().unwrap();
    if error(dyon_runtime.run(&dyon_module)) {
        return
    }
}

fn load_module() -> Option<Module> {
    let mut module = Module::new();
    module.add(Arc::new("say_hello".into()), say_hello, PreludeFunction {
        lts: vec![],
        returns: false
    });
    module.add(Arc::new("homer".into()), homer, PreludeFunction {
        lts: vec![],
        returns: true
    });
    module.add(Arc::new("age".into()), age, PreludeFunction {
        lts: vec![Lt::Default],
        returns: true
    });
    if error(load("source/functions/loader.rs", &mut module)) {
        None
    } else {
        Some(module)
    }
}

fn say_hello(_: &mut Runtime) -> Result<(), String> {
    println!("hi!");
    Ok(())
}

fn homer(rt: &mut Runtime) -> Result<(), String> {
    rt.push(Person { first_name: "Homer".into(), last_name: "Simpson".into(), age: 48 });
    Ok(())
}

fn age(rt: &mut Runtime) -> Result<(), String> {
    let mut person: Person = try!(rt.pop());
    person.age += 1;
    rt.push(person);
    Ok(())
}

pub struct Person {
    pub first_name: String,
    pub last_name: String,
    pub age: u32,
}

impl embed::PopVariable for Person {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        use dyon::embed::obj_field;
        let var = rt.resolve(var);
        if let &Variable::Object(ref obj) = var {
            Ok(Person {
                first_name: try!(obj_field(rt, obj, "first_name")),
                last_name: try!(obj_field(rt, obj, "last_name")),
                age: try!(obj_field(rt, obj, "age")),
            })
        } else {
            Err(rt.expected(var, "Person"))
        }
    }
}

impl embed::PushVariable for Person {
    fn push_var(&self) -> Variable {
        let mut obj: HashMap<_, Variable> = HashMap::new();
        obj.insert(Arc::new("first_name".into()), self.first_name.push_var());
        obj.insert(Arc::new("last_name".into()), self.last_name.push_var());
        obj.insert(Arc::new("age".into()), (self.age as f64).push_var());
        Variable::Object(Arc::new(obj))
    }
}
