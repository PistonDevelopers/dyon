use std::sync::Arc;

use runtime::{Flow, Runtime, Side};
use ast;
use prelude::{Lt, Prelude, Dfn};

use Module;
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
const LOAD: usize = 14;
const LOAD__SOURCE_IMPORTS: usize = 15;
const CALL: usize = 16;
const CALL_RET: usize = 17;
const FUNCTIONS: usize = 18;
const UNWRAP: usize = 19;
const UNWRAP_ERR: usize = 20;
const IS_ERR: usize = 21;
const IS_OK: usize = 22;
const MIN: usize = 23;
const MAX: usize = 24;
const SAVE__DATA_FILE: usize = 25;
const JSON_FROM_META_DATA: usize = 26;
const HAS: usize = 27;
const CHARS: usize = 28;
const UNWRAP_OR: usize = 29;
const TIP: usize = 30;
const NECK: usize = 31;
const FUNCTIONS__MODULE: usize = 32;
const KEYS: usize = 33;
const ERRSTR__STRING_START_LEN_MSG: usize = 34;
const META__SYNTAX_IN_STRING: usize = 35;
const MODULE__IN_STRING_IMPORTS: usize = 36;
const INSERT: usize = 37;
const INSERT_REF: usize = 38;
const REMOVE: usize = 39;
const NEXT: usize = 40;
const WAIT_NEXT: usize = 41;

const TABLE: &'static [(usize, fn(
        &mut Runtime,
        &ast::Call,
        &Arc<Module>,
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
    (LOAD, load),
    (LOAD__SOURCE_IMPORTS, load__source_imports),
    (CALL, _call),
    (CALL_RET, call_ret),
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
    (MODULE__IN_STRING_IMPORTS, module__in_string_imports),
    (INSERT, insert),
    (INSERT_REF, insert_ref),
    (REMOVE, remove),
    (NEXT, next),
    (WAIT_NEXT, wait_next),
];

pub fn standard(f: &mut Prelude) {
    let sarg = |f: &mut Prelude, name: &str, index: usize, ty: Type, ret: Type| {
        f.intrinsic(Arc::new(name.into()), index, Dfn {
            lts: vec![Lt::Default],
            tys: vec![ty],
            ret: ret
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
    sarg(f, "load", LOAD, Type::Text, Type::result());
    f.intrinsic(Arc::new("load__source_imports".into()), LOAD__SOURCE_IMPORTS, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Text, Type::array()],
        ret: Type::result()
    });
    f.intrinsic(Arc::new("call".into()), CALL, Dfn {
        lts: vec![Lt::Default; 3],
        tys: vec![Type::Any, Type::Text, Type::array()],
        ret: Type::Void
    });
    f.intrinsic(Arc::new("call_ret".into()), CALL_RET, Dfn {
        lts: vec![Lt::Default; 3],
        tys: vec![Type::Any, Type::Text, Type::array()],
        ret: Type::Any
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
    f.intrinsic(Arc::new("module__in_string_imports".into()), MODULE__IN_STRING_IMPORTS, Dfn {
        lts: vec![Lt::Default; 3],
        tys: vec![Type::Text, Type::Text, Type::array()],
        ret: Type::result()
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

pub fn call_standard(
    rt: &mut Runtime,
    index: usize,
    call: &ast::Call,
    module: &Arc<Module>
) -> Result<(Option<Variable>, Flow), String> {
    for arg in &call.args {
        match try!(rt.expression(arg, Side::Right, module)) {
            (x, Flow::Return) => { return Ok((x, Flow::Return)); }
            (Some(v), Flow::Continue) => rt.stack.push(v),
            _ => return Err(module.error(arg.source_range(),
                    &format!("{}\nExpected something. \
                    Expression did not return a value.",
                    rt.stack_trace()), rt))
        };
    }
    let (ind, f) = TABLE[index];
    debug_assert!(ind == index);
    let expect = try!((f)(rt, call, module));
    Ok((expect, Flow::Continue))
}
