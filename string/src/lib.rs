#[macro_use]
extern crate dyon;

use std::sync::Arc;
use dyon::{Dfn, Lt, Type, Module, Variable};

/// Adds string functions to module.
pub fn add_functions(module: &mut Module) {
    module.add(Arc::new("lines".into()), lines, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::Text],
        ret: Type::Array(Box::new(Type::Text)),
    });
}

dyon_fn!{fn lines(text: Arc<String>) -> Variable {
    let mut arr = vec![];
    for line in text.lines() {
        arr.push(Variable::Text(Arc::new(line.into())));
    }
    Variable::Array(Arc::new(arr))
}}
