use std::sync::Arc;

use runtime::{Flow, Runtime, Side};
use ast;
use prelude::{Lt, Prelude, Dfn};

use Variable;
use Type;
use dyon_std::*;

const WHY: usize = 0;
const WHERE: usize = 1;
const EXPLAIN_WHY: usize = 2;
const EXPLAIN_WHERE: usize = 3;
const HEAD: usize = 4;
const TAIL: usize = 5;
const IS_EMPTY: usize = 6;
const LEN: usize = 7;
const PUSH_REF: usize = 8;
const PUSH: usize = 9;
const POP: usize = 10;
const REVERSE: usize = 11;
const CLEAR: usize = 12;
const SWAP: usize = 13;
const FUNCTIONS: usize = 14;
const UNWRAP: usize = 15;
const UNWRAP_ERR: usize = 16;
const IS_ERR: usize = 17;
const IS_OK: usize = 18;
const MIN: usize = 19;
const MAX: usize = 20;
const SAVE__DATA_FILE: usize = 21;
const JSON_FROM_META_DATA: usize = 22;
const HAS: usize = 23;
const CHARS: usize = 24;
const UNWRAP_OR: usize = 25;
const TIP: usize = 26;
const NECK: usize = 27;
const FUNCTIONS__MODULE: usize = 28;
const KEYS: usize = 29;
const ERRSTR__STRING_START_LEN_MSG: usize = 30;
const META__SYNTAX_IN_STRING: usize = 31;
const INSERT: usize = 32;
const INSERT_REF: usize = 33;
const REMOVE: usize = 34;
const NEXT: usize = 35;
const WAIT_NEXT: usize = 36;

const TABLE: &[(usize, fn(
        &mut Runtime,
        &ast::Call,
    ) -> Result<Option<Variable>, String>)]
= &[
    (WHY, why),
    (WHERE, _where),
    (EXPLAIN_WHY, explain_why),
    (EXPLAIN_WHERE, explain_where),
    (HEAD, head),
    (TAIL, tail),
    (IS_EMPTY, is_empty),
    (LEN, len),
    (PUSH_REF, push_ref),
    (PUSH, push),
    (POP, pop),
    (REVERSE, reverse),
    (CLEAR, clear),
    (SWAP, swap),
    (FUNCTIONS, functions),
    (UNWRAP, unwrap),
    (UNWRAP_ERR, unwrap_err),
    (IS_ERR, is_err),
    (IS_OK, is_ok),
    (MIN, min),
    (MAX, max),
    (SAVE__DATA_FILE, save__data_file),
    (JSON_FROM_META_DATA, json_from_meta_data),
    (HAS, has),
    (CHARS, chars),
    (UNWRAP_OR, unwrap_or),
    (TIP, tip),
    (NECK, neck),
    (FUNCTIONS__MODULE, functions__module),
    (KEYS, keys),
    (ERRSTR__STRING_START_LEN_MSG, errstr__string_start_len_msg),
    (META__SYNTAX_IN_STRING, meta__syntax_in_string),
    (INSERT, insert),
    (INSERT_REF, insert_ref),
    (REMOVE, remove),
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

    f.intrinsic(Arc::new("why".into()), WHY, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::Secret(Box::new(Type::Bool))],
        ret: Type::array()
    });
    f.intrinsic(Arc::new("where".into()), WHERE, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::Secret(Box::new(Type::F64))],
        ret: Type::array()
    });
    f.intrinsic(Arc::new("explain_why".into()), EXPLAIN_WHY, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Bool, Type::Any],
        ret: Type::Secret(Box::new(Type::Bool))
    });
    f.intrinsic(Arc::new("explain_where".into()), EXPLAIN_WHERE, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::F64, Type::Any],
        ret: Type::Secret(Box::new(Type::F64))
    });
    sarg(f, "head", HEAD, Type::Link, Type::Any);
    sarg(f, "tail", TAIL, Type::Link, Type::Link);
    sarg(f, "is_empty", IS_EMPTY, Type::Link, Type::Bool);
    sarg(f, "len", LEN, Type::array(), Type::F64);
    f.intrinsic(Arc::new("push_ref(mut,_)".into()), PUSH_REF, Dfn {
        lts: vec![Lt::Default, Lt::Arg(0)],
        tys: vec![Type::array(), Type::Any],
        ret: Type::Void
    });
    f.intrinsic(Arc::new("push(mut,_)".into()), PUSH, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::array(), Type::Any],
        ret: Type::Void
    });
    f.intrinsic(Arc::new("pop(mut)".into()), POP, Dfn {
        lts: vec![Lt::Return],
        tys: vec![Type::array()],
        ret: Type::Any
    });
    sarg(f, "reverse(mut)", REVERSE, Type::array(), Type::Void);
    sarg(f, "clear(mut)", CLEAR, Type::array(), Type::Void);
    f.intrinsic(Arc::new("swap(mut,_,_)".into()), SWAP, Dfn {
        lts: vec![Lt::Default; 3],
        tys: vec![Type::array(), Type::F64, Type::F64],
        ret: Type::Void
    });
    f.intrinsic(Arc::new("functions".into()), FUNCTIONS, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::Any
    });
    sarg(f, "unwrap", UNWRAP, Type::Any, Type::Any);
    sarg(f, "unwrap_err", UNWRAP_ERR, Type::Any, Type::Any);
    sarg(f, "is_err", IS_ERR, Type::result(), Type::Bool);
    sarg(f, "is_ok", IS_OK, Type::result(), Type::Bool);
    sarg(f, "min", MIN, Type::Array(Box::new(Type::F64)), Type::F64);
    sarg(f, "max", MAX, Type::Array(Box::new(Type::F64)), Type::F64);
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
    f.intrinsic(Arc::new("unwrap_or".into()), UNWRAP_OR, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Any, Type::Any],
        ret: Type::Any
    });
    sarg(f, "tip", TIP, Type::Link, Type::Option(Box::new(Type::Any)));
    sarg(f, "neck", NECK, Type::Link, Type::Link);
    sarg(f, "functions__module", FUNCTIONS__MODULE, Type::Any, Type::Any);
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
    f.intrinsic(Arc::new("insert(mut,_,_)".into()), INSERT, Dfn {
        lts: vec![Lt::Default; 3],
        tys: vec![Type::array(), Type::F64, Type::Any],
        ret: Type::Void
    });
    f.intrinsic(Arc::new("insert_ref(mut,_,_)".into()), INSERT_REF, Dfn {
        lts: vec![Lt::Default, Lt::Default, Lt::Arg(0)],
        tys: vec![Type::array(), Type::F64, Type::Any],
        ret: Type::Void
    });
    f.intrinsic(Arc::new("remove(mut,_)".into()), REMOVE, Dfn {
        lts: vec![Lt::Return, Lt::Default],
        tys: vec![Type::array(), Type::F64],
        ret: Type::Any
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
