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
    module.add(Arc::new("ends_with".into()), ends_with, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Text; 2],
        ret: Type::Bool,
    });
    module.add(Arc::new("to_lowercase".into()), to_lowercase, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::Text],
        ret: Type::Text,
    });
    module.add(Arc::new("to_uppercase".into()), to_uppercase, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::Text],
        ret: Type::Text,
    });
    module.add(Arc::new("is_ascii".into()), is_ascii, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::Text],
        ret: Type::Bool,
    });
    module.add(Arc::new("to_ascii_lowercase".into()), to_ascii_lowercase, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::Text],
        ret: Type::Text,
    });
    module.add(Arc::new("to_ascii_uppercase".into()), to_ascii_uppercase, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::Text],
        ret: Type::Text,
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

dyon_fn!{fn ends_with(text: Arc<String>, pat: Arc<String>) -> bool {
    text.ends_with(&**pat)
}}

dyon_fn!{fn to_lowercase(text: Arc<String>) -> Arc<String> {
    Arc::new(text.to_lowercase())
}}

dyon_fn!{fn to_uppercase(text: Arc<String>) -> Arc<String> {
    Arc::new(text.to_uppercase())
}}

dyon_fn!{fn is_ascii(text: Arc<String>) -> bool {
    text.is_ascii()
}}

dyon_fn!{fn to_ascii_lowercase(text: Arc<String>) -> Arc<String> {
    Arc::new(text.to_ascii_lowercase())
}}

dyon_fn!{fn to_ascii_uppercase(text: Arc<String>) -> Arc<String> {
    Arc::new(text.to_ascii_uppercase())
}}
