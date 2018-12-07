use std::sync::Arc;

use runtime::{Flow, Runtime, Side};
use ast;
use prelude::{Lt, Prelude, Dfn};

use Module;
use Variable;
use Type;
use dyon_std::*;

const X: usize = 0;
const Y: usize = 1;
const Z: usize = 2;
const W: usize = 3;
const WHY: usize = 4;
const WHERE: usize = 5;
const EXPLAIN_WHY: usize = 6;
const EXPLAIN_WHERE: usize = 7;
const PRINTLN: usize = 8;
const PRINT: usize = 9;
const CLONE: usize = 10;
const DEBUG: usize = 11;
const BACKTRACE: usize = 12;
const SLEEP: usize = 13;
const RANDOM: usize = 14;
const HEAD: usize = 15;
const TAIL: usize = 16;
const IS_EMPTY: usize = 17;
const READ_NUMBER: usize = 18;
const READ_LINE: usize = 19;
const LEN: usize = 20;
const PUSH_REF: usize = 21;
const PUSH: usize = 22;
const POP: usize = 23;
const REVERSE: usize = 24;
const CLEAR: usize = 25;
const SWAP: usize = 26;
const TRIM: usize = 27;
const TRIM_LEFT: usize = 28;
const TRIM_RIGHT: usize = 29;
const STR: usize = 30;
const JSON_STRING: usize = 31;
const STR__COLOR: usize = 32;
const SRGB_TO_LINEAR__COLOR: usize = 33;
const LINEAR_TO_SRGB__COLOR: usize = 34;
const TYPEOF: usize = 35;
const ROUND: usize = 36;
const ABS: usize = 37;
const FLOOR: usize = 38;
const CEIL: usize = 39;
const SQRT: usize = 40;
const SIN: usize = 41;
const ASIN: usize = 42;
const COS: usize = 43;
const ACOS: usize = 44;
const TAN: usize = 45;
const ATAN: usize = 46;
const EXP: usize = 47;
const LN: usize = 48;
const LOG2: usize = 49;
const LOG10: usize = 50;
const LOAD: usize = 51;
const LOAD__SOURCE_IMPORTS: usize = 52;
const CALL: usize = 53;
const CALL_RET: usize = 54;
const FUNCTIONS: usize = 55;
const NONE: usize = 56;
const SOME: usize = 57;
const UNWRAP: usize = 58;
const UNWRAP_ERR: usize = 59;
const OK: usize = 60;
const ERR: usize = 61;
const IS_ERR: usize = 62;
const IS_OK: usize = 63;
const MIN: usize = 64;
const MAX: usize = 65;
const S: usize = 66;
const DIR__ANGLE: usize = 67;
const LOAD__META_FILE: usize = 68;
const LOAD__META_URL: usize = 69;
const DOWNLOAD__URL_FILE: usize = 70;
const SAVE__STRING_FILE: usize = 71;
const LOAD_STRING__FILE: usize = 72;
const JOIN__THREAD: usize = 73;
const SAVE__DATA_FILE: usize = 74;
const JSON_FROM_META_DATA: usize = 75;
const HAS: usize = 76;
const CHARS: usize = 77;
const NOW: usize = 78;
const IS_NAN: usize = 79;
const ATAN2: usize = 80;
const UNWRAP_OR: usize = 81;
const TIP: usize = 82;
const NECK: usize = 83;
const LOAD_DATA__FILE: usize = 84;
const FUNCTIONS__MODULE: usize = 85;
const KEYS: usize = 86;
const ERRSTR__STRING_START_LEN_MSG: usize = 87;
const SYNTAX__IN_STRING: usize = 88;
const META__SYNTAX_IN_STRING: usize = 89;
const MODULE__IN_STRING_IMPORTS: usize = 90;
const LOAD_STRING__URL: usize = 91;
const PARSE_NUMBER: usize = 92;
const INSERT: usize = 93;
const INSERT_REF: usize = 94;
const REMOVE: usize = 95;
const NEXT: usize = 96;
const WAIT_NEXT: usize = 97;
const LOAD_DATA__STRING: usize = 98;
const ARGS_OS: usize = 99;
const RX: usize = 100;
const RY: usize = 101;
const RZ: usize = 102;
const RW: usize = 103;
const RV: usize = 104;
const CX: usize = 105;
const CY: usize = 106;
const CZ: usize = 107;
const CW: usize = 108;
const CV: usize = 109;
const DET: usize = 110;
const INV: usize = 111;
const MOV: usize = 112;
const ROT__AXIS_ANGLE: usize = 113;
const TAU: usize = 114;

const TABLE: &'static [(usize, fn(
        &mut Runtime,
        &ast::Call,
        &Arc<Module>,
    ) -> Result<Option<Variable>, String>)]
= &[
    (X, x),
    (Y, y),
    (Z, z),
    (W, w),
    (WHY, why),
    (WHERE, _where),
    (EXPLAIN_WHY, explain_why),
    (EXPLAIN_WHERE, explain_where),
    (PRINTLN, println),
    (PRINT, print),
    (CLONE, clone),
    (DEBUG, debug),
    (BACKTRACE, backtrace),
    (SLEEP, sleep),
    (RANDOM, random),
    (HEAD, head),
    (TAIL, tail),
    (IS_EMPTY, is_empty),
    (READ_NUMBER, read_number),
    (READ_LINE, read_line),
    (LEN, len),
    (PUSH_REF, push_ref),
    (PUSH, push),
    (POP, pop),
    (REVERSE, reverse),
    (CLEAR, clear),
    (SWAP, swap),
    (TRIM, trim),
    (TRIM_LEFT, trim_left),
    (TRIM_RIGHT, trim_right),
    (STR, _str),
    (JSON_STRING, json_string),
    (STR__COLOR, str__color),
    (SRGB_TO_LINEAR__COLOR, srgb_to_linear__color),
    (LINEAR_TO_SRGB__COLOR, linear_to_srgb__color),
    (TYPEOF, _typeof),
    (ROUND, round),
    (ABS, abs),
    (FLOOR, floor),
    (CEIL, ceil),
    (SQRT, sqrt),
    (SIN, sin),
    (ASIN, asin),
    (COS, cos),
    (ACOS, acos),
    (TAN, tan),
    (ATAN, atan),
    (EXP, exp),
    (LN, ln),
    (LOG2, log2),
    (LOG10, log10),
    (LOAD, load),
    (LOAD__SOURCE_IMPORTS, load__source_imports),
    (CALL, _call),
    (CALL_RET, call_ret),
    (FUNCTIONS, functions),
    (NONE, none),
    (SOME, some),
    (UNWRAP, unwrap),
    (UNWRAP_ERR, unwrap_err),
    (OK, ok),
    (ERR, err),
    (IS_ERR, is_err),
    (IS_OK, is_ok),
    (MIN, min),
    (MAX, max),
    (S, s),
    (DIR__ANGLE, dir__angle),
    (LOAD__META_FILE, load__meta_file),
    (LOAD__META_URL, load__meta_url),
    (DOWNLOAD__URL_FILE, download__url_file),
    (SAVE__STRING_FILE, save__string_file),
    (LOAD_STRING__FILE, load_string__file),
    (JOIN__THREAD, join__thread),
    (SAVE__DATA_FILE, save__data_file),
    (JSON_FROM_META_DATA, json_from_meta_data),
    (HAS, has),
    (CHARS, chars),
    (NOW, now),
    (IS_NAN, is_nan),
    (ATAN2, atan2),
    (UNWRAP_OR, unwrap_or),
    (TIP, tip),
    (NECK, neck),
    (LOAD_DATA__FILE, load_data__file),
    (FUNCTIONS__MODULE, functions__module),
    (KEYS, keys),
    (ERRSTR__STRING_START_LEN_MSG, errstr__string_start_len_msg),
    (SYNTAX__IN_STRING, syntax__in_string),
    (META__SYNTAX_IN_STRING, meta__syntax_in_string),
    (MODULE__IN_STRING_IMPORTS, module__in_string_imports),
    (LOAD_STRING__URL, load_string__url),
    (PARSE_NUMBER, parse_number),
    (INSERT, insert),
    (INSERT_REF, insert_ref),
    (REMOVE, remove),
    (NEXT, next),
    (WAIT_NEXT, wait_next),
    (LOAD_DATA__STRING, load_data__string),
    (ARGS_OS, args_os),
    (RX, rx),
    (RY, ry),
    (RZ, rz),
    (RW, rw),
    (RV, rv),
    (CX, cx),
    (CY, cy),
    (CZ, cz),
    (CW, cw),
    (CV, cv),
    (DET, det),
    (INV, inv),
    (MOV, mov),
    (ROT__AXIS_ANGLE, rot__axis_angle),
    (TAU, tau),
];

pub fn standard(f: &mut Prelude) {
    let sarg = |f: &mut Prelude, name: &str, index: usize, ty: Type, ret: Type| {
        f.intrinsic(Arc::new(name.into()), index, Dfn {
            lts: vec![Lt::Default],
            tys: vec![ty],
            ret: ret
        });
    };

    sarg(f, "x", X, Type::Vec4, Type::F64);
    sarg(f, "y", Y, Type::Vec4, Type::F64);
    sarg(f, "z", Z, Type::Vec4, Type::F64);
    sarg(f, "w", W, Type::Vec4, Type::F64);
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
    sarg(f, "println", PRINTLN, Type::Any, Type::Void);
    sarg(f, "print", PRINT, Type::Any, Type::Void);
    sarg(f, "clone", CLONE, Type::Any, Type::Any);
    f.intrinsic(Arc::new("debug".into()), DEBUG, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::Void
    });
    f.intrinsic(Arc::new("backtrace".into()), BACKTRACE, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::Void
    });
    sarg(f, "sleep", SLEEP, Type::F64, Type::Void);
    f.intrinsic(Arc::new("random".into()), RANDOM, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::F64
    });
    sarg(f, "head", HEAD, Type::Link, Type::Any);
    sarg(f, "tail", TAIL, Type::Link, Type::Link);
    sarg(f, "is_empty", IS_EMPTY, Type::Link, Type::Bool);
    sarg(f, "read_number", READ_NUMBER, Type::Text, Type::F64);
    f.intrinsic(Arc::new("read_line".into()), READ_LINE, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::Text
    });
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
    sarg(f, "trim", TRIM, Type::Text, Type::Text);
    sarg(f, "trim_left", TRIM_LEFT, Type::Text, Type::Text);
    sarg(f, "trim_right", TRIM_RIGHT, Type::Text, Type::Text);
    sarg(f, "str", STR, Type::Any, Type::Text);
    sarg(f, "json_string", JSON_STRING, Type::Text, Type::Text);
    sarg(f, "str__color", STR__COLOR, Type::Vec4, Type::Text);
    sarg(f, "srgb_to_linear__color", SRGB_TO_LINEAR__COLOR, Type::Vec4, Type::Vec4);
    sarg(f, "linear_to_srgb__color", LINEAR_TO_SRGB__COLOR, Type::Vec4, Type::Vec4);
    sarg(f, "typeof", TYPEOF, Type::Any, Type::Text);
    sarg(f, "round", ROUND, Type::F64, Type::F64);
    sarg(f, "abs", ABS, Type::F64, Type::F64);
    sarg(f, "floor", FLOOR, Type::F64, Type::F64);
    sarg(f, "ceil", CEIL, Type::F64, Type::F64);
    sarg(f, "sqrt", SQRT, Type::F64, Type::F64);
    sarg(f, "sin", SIN, Type::F64, Type::F64);
    sarg(f, "asin", ASIN, Type::F64, Type::F64);
    sarg(f, "cos", COS, Type::F64, Type::F64);
    sarg(f, "acos", ACOS, Type::F64, Type::F64);
    sarg(f, "tan", TAN, Type::F64, Type::F64);
    sarg(f, "atan", ATAN, Type::F64, Type::F64);
    sarg(f, "exp", EXP, Type::F64, Type::F64);
    sarg(f, "ln", LN, Type::F64, Type::F64);
    sarg(f, "log2", LOG2, Type::F64, Type::F64);
    sarg(f, "log10", LOG10, Type::F64, Type::F64);
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
    f.intrinsic(Arc::new("none".into()), NONE, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::option()
    });
    sarg(f, "some", SOME, Type::Any, Type::option());
    sarg(f, "unwrap", UNWRAP, Type::Any, Type::Any);
    sarg(f, "unwrap_err", UNWRAP_ERR, Type::Any, Type::Any);
    sarg(f, "ok", OK, Type::Any, Type::result());
    sarg(f, "err", ERR, Type::Any, Type::result());
    sarg(f, "is_err", IS_ERR, Type::result(), Type::Bool);
    sarg(f, "is_ok", IS_OK, Type::result(), Type::Bool);
    sarg(f, "min", MIN, Type::Array(Box::new(Type::F64)), Type::F64);
    sarg(f, "max", MAX, Type::Array(Box::new(Type::F64)), Type::F64);
    f.intrinsic(Arc::new("s".into()), S, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Vec4, Type::F64],
        ret: Type::F64
    });
    sarg(f, "dir__angle", DIR__ANGLE, Type::F64, Type::Vec4);
    f.intrinsic(Arc::new("load__meta_file".into()), LOAD__META_FILE, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Text; 2],
        ret: Type::Result(Box::new(Type::Array(Box::new(Type::array()))))
    });
    f.intrinsic(Arc::new("load__meta_url".into()), LOAD__META_URL, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Text; 2],
        ret: Type::Result(Box::new(Type::Array(Box::new(Type::array()))))
    });
    f.intrinsic(Arc::new("download__url_file".into()), DOWNLOAD__URL_FILE, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Text; 2],
        ret: Type::Result(Box::new(Type::Text))
    });
    f.intrinsic(Arc::new("save__string_file".into()), SAVE__STRING_FILE, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Text; 2],
        ret: Type::Result(Box::new(Type::Text))
    });
    sarg(f, "load_string__file", LOAD_STRING__FILE, Type::Text, Type::Result(Box::new(Type::Text)));
    sarg(f, "join__thread", JOIN__THREAD, Type::thread(), Type::Result(Box::new(Type::Any)));
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
    f.intrinsic(Arc::new("now".into()), NOW, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::F64
    });
    sarg(f, "is_nan", IS_NAN, Type::F64, Type::Bool);
    f.intrinsic(Arc::new("atan2".into()), ATAN2, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::F64; 2],
        ret: Type::F64
    });
    f.intrinsic(Arc::new("unwrap_or".into()), UNWRAP_OR, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Any, Type::Any],
        ret: Type::Any
    });
    sarg(f, "tip", TIP, Type::Link, Type::Option(Box::new(Type::Any)));
    sarg(f, "neck", NECK, Type::Link, Type::Link);
    sarg(f, "load_data__file", LOAD_DATA__FILE, Type::Text, Type::Result(Box::new(Type::Any)));
    sarg(f, "functions__module", FUNCTIONS__MODULE, Type::Any, Type::Any);
    sarg(f, "keys", KEYS, Type::Object, Type::Array(Box::new(Type::Text)));
    f.intrinsic(Arc::new("errstr__string_start_len_msg".into()),
        ERRSTR__STRING_START_LEN_MSG, Dfn {
            lts: vec![Lt::Default; 4],
            tys: vec![Type::Text, Type::F64, Type::F64, Type::Text],
            ret: Type::Text
        });
    f.intrinsic(Arc::new("syntax__in_string".into()),
        SYNTAX__IN_STRING, Dfn {
            lts: vec![Lt::Default; 2],
            tys: vec![Type::Text; 2],
            ret: Type::Result(Box::new(Type::Any))
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
    sarg(f, "load_string__url", LOAD_STRING__URL, Type::Text, Type::Result(Box::new(Type::Text)));
    sarg(f, "parse_number", PARSE_NUMBER, Type::Text, Type::Option(Box::new(Type::F64)));
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
    sarg(f, "load_data__string", LOAD_DATA__STRING, Type::Text, Type::Result(Box::new(Type::Any)));
    f.intrinsic(Arc::new("args_os".into()), ARGS_OS, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::Array(Box::new(Type::Text))
    });
    sarg(f, "rx", RX, Type::Mat4, Type::Vec4);
    sarg(f, "ry", RY, Type::Mat4, Type::Vec4);
    sarg(f, "rz", RZ, Type::Mat4, Type::Vec4);
    sarg(f, "rw", RW, Type::Mat4, Type::Vec4);
    f.intrinsic(Arc::new("rv".into()), RV, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Mat4, Type::F64],
        ret: Type::Vec4
    });
    sarg(f, "cx", CX, Type::Mat4, Type::Vec4);
    sarg(f, "cy", CY, Type::Mat4, Type::Vec4);
    sarg(f, "cz", CZ, Type::Mat4, Type::Vec4);
    sarg(f, "cw", CW, Type::Mat4, Type::Vec4);
    f.intrinsic(Arc::new("cv".into()), CV, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Mat4, Type::F64],
        ret: Type::Vec4
    });
    sarg(f, "det", DET, Type::Mat4, Type::F64);
    sarg(f, "inv", INV, Type::Mat4, Type::Mat4);
    sarg(f, "mov", MOV, Type::Vec4, Type::Mat4);
    f.intrinsic(Arc::new("rot__axis_angle".into()), ROT__AXIS_ANGLE, Dfn {
        lts: vec![Lt::Default; 2],
        tys: vec![Type::Vec4, Type::F64],
        ret: Type::Mat4
    });
    f.intrinsic(Arc::new("tau".into()), TAU, Dfn {
        lts: vec![],
        tys: vec![],
        ret: Type::F64
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
