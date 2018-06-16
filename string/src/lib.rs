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
    module.add(Arc::new("split".into()), split, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Text, Type::Array(Box::new(Type::Text))],
        ret: Type::Array(Box::new(Type::Text)),
    });
    module.add(Arc::new("starts_with".into()), starts_with, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Text; 2],
        ret: Type::Bool,
    });
}

dyon_fn!{fn lines(text: Arc<String>) -> Variable {
    let mut arr = vec![];
    for line in text.lines() {
        arr.push(Variable::Text(Arc::new(line.into())));
    }
    Variable::Array(Arc::new(arr))
}}

dyon_fn!{fn split(text: Arc<String>, chs: Variable) -> Variable {
    let mut arr = vec![];
    if let Variable::Array(ref chs) = chs {
        for line in text.split(|c| chs.iter().any(|v|
            if let Variable::Text(ref txt) = *v {
                if txt.chars().next() == Some(c) {true}
                else {false}
            } else {false}
        )) {
            arr.push(Variable::Text(Arc::new(line.into())));
        }
    }
    Variable::Array(Arc::new(arr))
}}

dyon_fn!{fn starts_with(text: Arc<String>, pat: Arc<String>) -> bool {
    text.starts_with(&**pat)
}}
