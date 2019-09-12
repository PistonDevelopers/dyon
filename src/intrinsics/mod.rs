use std::sync::Arc;

use runtime::{Flow, Runtime, Side};
use ast;
use prelude::{Lt, Prelude, Dfn};

use Variable;
use Type;
use dyon_std::*;

const UNWRAP_ERR: usize = 0;
const SAVE__DATA_FILE: usize = 1;
const JSON_FROM_META_DATA: usize = 2;
const HAS: usize = 3;
const CHARS: usize = 4;
const KEYS: usize = 5;
const ERRSTR__STRING_START_LEN_MSG: usize = 6;
const META__SYNTAX_IN_STRING: usize = 7;
const NEXT: usize = 8;
const WAIT_NEXT: usize = 9;

const TABLE: &[(usize, fn(
        &mut Runtime,
        &ast::Call,
    ) -> Result<Option<Variable>, String>)]
= &[
    (UNWRAP_ERR, unwrap_err),
    (SAVE__DATA_FILE, save__data_file),
    (JSON_FROM_META_DATA, json_from_meta_data),
    (HAS, has),
    (CHARS, chars),
    (KEYS, keys),
    (ERRSTR__STRING_START_LEN_MSG, errstr__string_start_len_msg),
    (META__SYNTAX_IN_STRING, meta__syntax_in_string),
    (NEXT, next),
    (WAIT_NEXT, wait_next),
];

pub(crate) fn standard(f: &mut Prelude) {
    let sarg = |f: &mut Prelude, name: &str, index: usize, ty: Type, ret: Type| {
        f.intrinsic(Arc::new(name.into()), index, Dfn {
            lts: vec![Lt::Default],
            tys: vec![ty],
            ret
        });
    };

    sarg(f, "unwrap_err", UNWRAP_ERR, Type::Any, Type::Any);
    f.intrinsic(Arc::new("save__data_file".into()), SAVE__DATA_FILE, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Any, Type::Text],
        ret: Type::Result(Box::new(Type::Text))
    });
    sarg(f, "json_from_meta_data", JSON_FROM_META_DATA, Type::Array(Box::new(Type::array())), Type::Text);
    f.intrinsic(Arc::new("has".into()), HAS, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Object, Type::Text],
        ret: Type::Bool
    });
    sarg(f, "chars", CHARS, Type::Text, Type::Array(Box::new(Type::Text)));
    sarg(f, "keys", KEYS, Type::Object, Type::Array(Box::new(Type::Text)));
    f.intrinsic(Arc::new("errstr__string_start_len_msg".into()),
        ERRSTR__STRING_START_LEN_MSG, Dfn {
            lts: vec![Lt::Default; 4],
            tys: vec![Type::Text, Type::F64, Type::F64, Type::Text],
            ret: Type::Text
        });
    f.intrinsic(Arc::new("meta__syntax_in_string".into()),
        META__SYNTAX_IN_STRING, Dfn {
            lts: vec![Lt::Default; 3],
            tys: vec![Type::Any, Type::Text, Type::Text],
            ret: Type::Result(Box::new(Type::Array(Box::new(Type::array()))))
        });
    f.intrinsic(Arc::new("next".into()), NEXT, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::in_ty()],
        ret: Type::Any
    });
    f.intrinsic(Arc::new("wait_next".into()), WAIT_NEXT, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::in_ty()],
        ret: Type::Any
    });
}

pub(crate) fn call_standard(
    rt: &mut Runtime,
    index: usize,
    call: &ast::Call
) -> Result<(Option<Variable>, Flow), String> {
    for arg in &call.args {
        match rt.expression(arg, Side::Right)? {
            (x, Flow::Return) => { return Ok((x, Flow::Return)); }
            (Some(v), Flow::Continue) => rt.stack.push(v),
            _ => return Err(rt.module.error(arg.source_range(),
                    &format!("{}\nExpected something. \
                    Expression did not return a value.",
                    rt.stack_trace()), rt))
        };
    }
    let (ind, f) = TABLE[index];
    debug_assert!(ind == index);
    let expect = (f)(rt, call)?;
    Ok((expect, Flow::Continue))
}
