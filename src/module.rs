use super::*;

/// Stores functions for a Dyon module.
#[derive(Clone)]
pub struct Module {
    pub(crate) functions: Vec<ast::Function>,
    pub(crate) ext_prelude: Vec<FnExternal>,
    pub(crate) register_namespace: Arc<Vec<Arc<String>>>,
}

impl Default for Module {
    fn default() -> Module {
        Module::new()
    }
}

impl Module {
    /// Creates a new empty module.
    pub fn empty() -> Module {
        Module {
            functions: vec![],
            ext_prelude: vec![],
            register_namespace: Arc::new(vec![]),
        }
    }

    /// Import external prelude from other module.
    pub fn import_ext_prelude(&mut self, other: &Module) {
        for f in &other.ext_prelude {
            self.ext_prelude.push(f.clone());
        }
    }

    /// Import external prelude and loaded functions from module.
    pub fn import(&mut self, other: &Module) {
        // Add external functions from imports.
        for f in &other.ext_prelude {
            let has_external = self
                .ext_prelude
                .iter()
                .any(|a| a.name == f.name && a.namespace == f.namespace);
            if !has_external {
                self.ext_prelude.push(f.clone());
            }
        }
        // Register loaded functions from imports.
        for f in &other.functions {
            self.functions.push(f.clone())
        }
    }

    /// Creates a new module with standard library.
    pub fn new() -> Module {
        use dyon_std::*;
        use Type::*;

        let mut m = Module::empty();
        m.ns("std");
        m.add_binop(
            crate::LESS.clone(),
            less,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any, Any],
                ret: Bool,
                ext: vec![
                    (
                        vec![],
                        vec![Secret(Box::new(F64)), F64],
                        Secret(Box::new(Bool)),
                    ),
                    (vec![], vec![F64; 2], Bool),
                    (vec![], vec![Str; 2], Bool),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_binop(
            crate::LESS_OR_EQUAL.clone(),
            less_or_equal,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any, Any],
                ret: Bool,
                ext: vec![
                    (
                        vec![],
                        vec![Secret(Box::new(F64)), F64],
                        Secret(Box::new(Bool)),
                    ),
                    (vec![], vec![F64; 2], Bool),
                    (vec![], vec![Str; 2], Bool),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_binop(
            crate::GREATER.clone(),
            greater,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any, Any],
                ret: Bool,
                ext: vec![
                    (
                        vec![],
                        vec![Secret(Box::new(F64)), F64],
                        Secret(Box::new(Bool)),
                    ),
                    (vec![], vec![F64; 2], Bool),
                    (vec![], vec![Str; 2], Bool),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_binop(
            crate::GREATER_OR_EQUAL.clone(),
            greater_or_equal,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any, Any],
                ret: Bool,
                ext: vec![
                    (
                        vec![],
                        vec![Secret(Box::new(F64)), F64],
                        Secret(Box::new(Bool)),
                    ),
                    (vec![], vec![F64; 2], Bool),
                    (vec![], vec![Str; 2], Bool),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_binop(
            crate::EQUAL.clone(),
            equal,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any, Any],
                ret: Bool,
                ext: vec![
                    (
                        vec![],
                        vec![Secret(Box::new(F64)), F64],
                        Secret(Box::new(Bool)),
                    ),
                    (vec![], vec![F64; 2], Bool),
                    (vec![], vec![Str; 2], Bool),
                    (
                        vec![],
                        vec![Secret(Box::new(Bool)), Bool],
                        Secret(Box::new(Bool)),
                    ),
                    (vec![], vec![Bool; 2], Bool),
                    (vec![], vec![Vec4; 2], Bool),
                    (vec![], vec![Type::object(), Type::object()], Bool),
                    (vec![], vec![Type::array(), Type::array()], Bool),
                    (vec![], vec![Type::option(), Type::option()], Bool),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_binop(
            crate::NOT_EQUAL.clone(),
            not_equal,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any, Any],
                ret: Bool,
                ext: vec![
                    (
                        vec![],
                        vec![Secret(Box::new(F64)), F64],
                        Secret(Box::new(Bool)),
                    ),
                    (vec![], vec![F64; 2], Bool),
                    (vec![], vec![Str; 2], Bool),
                    (
                        vec![],
                        vec![Secret(Box::new(Bool)), Bool],
                        Secret(Box::new(Bool)),
                    ),
                    (vec![], vec![Bool; 2], Bool),
                    (vec![], vec![Vec4; 2], Bool),
                    (vec![], vec![Type::object(), Type::object()], Bool),
                    (vec![], vec![Type::array(), Type::array()], Bool),
                    (vec![], vec![Type::option(), Type::option()], Bool),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add(
            crate::AND_ALSO.clone(),
            and_also,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Bool, Bool],
                ret: Any,
                ext: vec![
                    (
                        vec![],
                        vec![Secret(Box::new(Bool)), Bool],
                        Secret(Box::new(Bool)),
                    ),
                    (vec![], vec![Bool; 2], Bool),
                ],
                lazy: LAZY_AND,
            },
        );
        m.add(
            crate::OR_ELSE.clone(),
            or_else,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Bool, Bool],
                ret: Any,
                ext: vec![
                    (
                        vec![],
                        vec![Secret(Box::new(Bool)), Bool],
                        Secret(Box::new(Bool)),
                    ),
                    (vec![], vec![Bool; 2], Bool),
                ],
                lazy: LAZY_OR,
            },
        );
        m.add_binop(
            crate::ADD.clone(),
            add,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any; 2],
                ret: Any,
                ext: vec![
                    Type::all_ext(vec![F64, F64], F64),
                    Type::all_ext(vec![Vec4, Vec4], Vec4),
                    Type::all_ext(vec![Vec4, F64], Vec4),
                    Type::all_ext(vec![F64, Vec4], Vec4),
                    Type::all_ext(vec![Mat4, Mat4], Mat4),
                    Type::all_ext(vec![F64, Mat4], Mat4),
                    Type::all_ext(vec![Mat4, F64], Mat4),
                    Type::all_ext(vec![Bool, Bool], Bool),
                    Type::all_ext(vec![Str, Str], Str),
                    Type::all_ext(vec![Link, Link], Link),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_binop(
            crate::SUB.clone(),
            sub,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any; 2],
                ret: Any,
                ext: vec![
                    Type::all_ext(vec![F64, F64], F64),
                    Type::all_ext(vec![Vec4, Vec4], Vec4),
                    Type::all_ext(vec![Vec4, F64], Vec4),
                    Type::all_ext(vec![F64, Vec4], Vec4),
                    Type::all_ext(vec![Mat4, Mat4], Mat4),
                    Type::all_ext(vec![F64, Mat4], Mat4),
                    Type::all_ext(vec![Mat4, F64], Mat4),
                    Type::all_ext(vec![Bool, Bool], Bool),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_binop(
            crate::MUL.clone(),
            mul,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any; 2],
                ret: Any,
                ext: vec![
                    (vec![], vec![F64, F64], F64),
                    (vec![], vec![Vec4, Vec4], Vec4),
                    (vec![], vec![Vec4, F64], Vec4),
                    (vec![], vec![F64, Vec4], Vec4),
                    (vec![], vec![Mat4, Mat4], Mat4),
                    (vec![], vec![F64, Mat4], Mat4),
                    (vec![], vec![Mat4, F64], Mat4),
                    (vec![], vec![Mat4, Vec4], Vec4),
                    Type::all_ext(vec![Bool, Bool], Bool),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_binop(
            crate::DIV.clone(),
            div,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any; 2],
                ret: Any,
                ext: vec![
                    (vec![], vec![F64, F64], F64),
                    (vec![], vec![Vec4, Vec4], Vec4),
                    (vec![], vec![Vec4, F64], Vec4),
                    (vec![], vec![F64, Vec4], Vec4),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_binop(
            crate::REM.clone(),
            rem,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any; 2],
                ret: Any,
                ext: vec![
                    (vec![], vec![F64, F64], F64),
                    (vec![], vec![Vec4, Vec4], Vec4),
                    (vec![], vec![Vec4, F64], Vec4),
                    (vec![], vec![F64, Vec4], Vec4),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_binop(
            crate::POW.clone(),
            pow,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any; 2],
                ret: Any,
                ext: vec![
                    (vec![], vec![F64, F64], F64),
                    (vec![], vec![Vec4, Vec4], Vec4),
                    (vec![], vec![Vec4, F64], Vec4),
                    (vec![], vec![F64, Vec4], Vec4),
                    Type::all_ext(vec![Bool, Bool], Bool),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_unop(
            crate::NOT.clone(),
            not,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Any],
                ret: Any,
                ext: vec![
                    (
                        vec![],
                        vec![Type::Secret(Box::new(Bool))],
                        Type::Secret(Box::new(Bool)),
                    ),
                    (vec![], vec![Bool], Bool),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_unop(
            crate::NEG.clone(),
            neg,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Any],
                ret: Any,
                ext: vec![
                    (vec![], vec![F64], F64),
                    (vec![], vec![Vec4], Vec4),
                    (vec![], vec![Mat4], Mat4),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_binop(
            crate::DOT.clone(),
            dot,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any; 2],
                ret: F64,
                ext: vec![
                    (vec![], vec![Vec4, Vec4], F64),
                    (vec![], vec![Vec4, F64], F64),
                    (vec![], vec![F64, Vec4], F64),
                ],
                lazy: LAZY_NO,
            },
        );
        m.add_str("cross", cross, Dfn::nl(vec![Vec4, Vec4], Vec4));
        m.add_str("x", x, Dfn::nl(vec![Vec4], F64));
        m.add_str("y", y, Dfn::nl(vec![Vec4], F64));
        m.add_str("z", z, Dfn::nl(vec![Vec4], F64));
        m.add_str("w", w, Dfn::nl(vec![Vec4], F64));
        m.add_unop_str("norm", norm, Dfn::nl(vec![Vec4], F64));
        m.add_str("det", det, Dfn::nl(vec![Mat4], F64));
        m.add_str("inv", inv, Dfn::nl(vec![Mat4], Mat4));
        m.add_str("mov", mov, Dfn::nl(vec![Vec4], Mat4));
        m.add_str(
            "rot__axis_angle",
            rot__axis_angle,
            Dfn::nl(vec![Vec4, F64], Mat4),
        );
        m.add_str(
            "ortho__pos_right_up_forward",
            ortho__pos_right_up_forward,
            Dfn::nl(vec![Vec4; 4], Mat4),
        );
        m.add_str(
            "proj__fov_near_far_ar",
            proj__fov_near_far_ar,
            Dfn::nl(vec![F64; 4], Mat4),
        );
        m.add_str(
            "mvp__model_view_projection",
            mvp__model_view_projection,
            Dfn::nl(vec![Mat4; 3], Mat4),
        );
        m.add_str("scale", scale, Dfn::nl(vec![Vec4], Mat4));
        m.add_str("rx", rx, Dfn::nl(vec![Mat4], Vec4));
        m.add_str("ry", ry, Dfn::nl(vec![Mat4], Vec4));
        m.add_str("rz", rz, Dfn::nl(vec![Mat4], Vec4));
        m.add_str("rw", rw, Dfn::nl(vec![Mat4], Vec4));
        m.add_str("cx", cx, Dfn::nl(vec![Mat4], Vec4));
        m.add_str("cy", cy, Dfn::nl(vec![Mat4], Vec4));
        m.add_str("cz", cz, Dfn::nl(vec![Mat4], Vec4));
        m.add_str("cw", cw, Dfn::nl(vec![Mat4], Vec4));
        m.add_str("cv", cv, Dfn::nl(vec![Mat4, F64], Vec4));
        m.add_str("clone", clone, Dfn::nl(vec![Any], Any));
        m.add_str("rv", rv, Dfn::nl(vec![Mat4, Type::F64], Vec4));
        m.add_str("s", s, Dfn::nl(vec![Vec4, F64], F64));
        #[cfg(feature = "stdio")]
        m.add_str("println", println, Dfn::nl(vec![Any], Void));
        #[cfg(feature = "stdio")]
        m.add_str("print", print, Dfn::nl(vec![Any], Void));
        m.add_str("sqrt", sqrt, Dfn::nl(vec![F64], F64));
        m.add_str("sin", sin, Dfn::nl(vec![F64], F64));
        m.add_str("asin", asin, Dfn::nl(vec![F64], F64));
        m.add_str("cos", cos, Dfn::nl(vec![F64], F64));
        m.add_str("acos", acos, Dfn::nl(vec![F64], F64));
        m.add_str("tan", tan, Dfn::nl(vec![F64], F64));
        m.add_str("atan", atan, Dfn::nl(vec![F64], F64));
        m.add_str("atan2", atan2, Dfn::nl(vec![F64; 2], F64));
        m.add_str("exp", exp, Dfn::nl(vec![F64], F64));
        m.add_str("ln", ln, Dfn::nl(vec![F64], F64));
        m.add_str("log2", log2, Dfn::nl(vec![F64], F64));
        m.add_str("log10", log10, Dfn::nl(vec![F64], F64));
        m.add_str("round", round, Dfn::nl(vec![F64], F64));
        m.add_str("abs", abs, Dfn::nl(vec![F64], F64));
        m.add_str("floor", floor, Dfn::nl(vec![F64], F64));
        m.add_str("ceil", ceil, Dfn::nl(vec![F64], F64));
        #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
        m.add_str("sleep", sleep, Dfn::nl(vec![F64], Void));
        #[cfg(feature = "rand")]
        m.add_str("random", random, Dfn::nl(vec![], F64));
        m.add_str("tau", tau, Dfn::nl(vec![], F64));
        #[cfg(feature = "stdio")]
        m.add_str("read_line", read_line, Dfn::nl(vec![], Str));
        #[cfg(feature = "stdio")]
        m.add_str("read_number", read_number, Dfn::nl(vec![Str], F64));
        m.add_str(
            "parse_number",
            parse_number,
            Dfn::nl(vec![Str], Option(Box::new(Type::F64))),
        );
        m.add_str("trim", trim, Dfn::nl(vec![Str], Str));
        m.add_str("trim_left", trim_left, Dfn::nl(vec![Str], Str));
        m.add_str("trim_right", trim_right, Dfn::nl(vec![Str], Str));
        m.add_str("str", _str, Dfn::nl(vec![Any], Str));
        m.add_str("json_string", json_string, Dfn::nl(vec![Str], Str));
        m.add_str("str__color", str__color, Dfn::nl(vec![Vec4], Str));
        m.add_str(
            "srgb_to_linear__color",
            srgb_to_linear__color,
            Dfn::nl(vec![Vec4], Vec4),
        );
        m.add_str(
            "linear_to_srgb__color",
            linear_to_srgb__color,
            Dfn::nl(vec![Vec4], Vec4),
        );
        m.add_str("typeof", _typeof, Dfn::nl(vec![Any], Str));
        m.add_str("debug", debug, Dfn::nl(vec![], Void));
        m.add_str("backtrace", backtrace, Dfn::nl(vec![], Void));
        m.add_str("none", none, Dfn::nl(vec![], Type::option()));
        m.add_str("some", some, Dfn::nl(vec![Any], Type::option()));
        m.add_str("ok", ok, Dfn::nl(vec![Any], Type::result()));
        m.add_str("err", err, Dfn::nl(vec![Any], Type::result()));
        m.add_str("dir__angle", dir__angle, Dfn::nl(vec![F64], Vec4));
        m.add_str(
            "load__meta_file",
            load__meta_file,
            Dfn::nl(
                vec![Str; 2],
                Type::Result(Box::new(Type::Array(Box::new(Type::array())))),
            ),
        );
        m.add_str(
            "load__meta_url",
            load__meta_url,
            Dfn::nl(
                vec![Str; 2],
                Type::Result(Box::new(Type::Array(Box::new(Type::array())))),
            ),
        );
        m.add_str(
            "syntax__in_string",
            syntax__in_string,
            Dfn::nl(vec![Type::Str; 2], Type::Result(Box::new(Any))),
        );
        m.add_str(
            "download__url_file",
            download__url_file,
            Dfn::nl(vec![Type::Str; 2], Type::Result(Box::new(Str))),
        );
        m.add_str(
            "save__string_file",
            save__string_file,
            Dfn::nl(vec![Type::Str; 2], Type::Result(Box::new(Str))),
        );
        m.add_str(
            "load_string__file",
            load_string__file,
            Dfn::nl(vec![Str], Type::Result(Box::new(Str))),
        );
        m.add_str(
            "load_string__url",
            load_string__url,
            Dfn::nl(vec![Str], Type::Result(Box::new(Str))),
        );
        #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
        m.add_str(
            "join__thread",
            join__thread,
            Dfn::nl(vec![Type::thread()], Type::Result(Box::new(Any))),
        );
        m.add_str(
            "load_data__file",
            load_data__file,
            Dfn::nl(vec![Str], Type::Result(Box::new(Any))),
        );
        m.add_str(
            "load_data__string",
            load_data__string,
            Dfn::nl(vec![Str], Type::Result(Box::new(Any))),
        );
        m.add_str(
            "args_os",
            args_os,
            Dfn::nl(vec![], Type::Array(Box::new(Str))),
        );
        m.add_str("now", now, Dfn::nl(vec![], F64));
        m.add_str("is_nan", is_nan, Dfn::nl(vec![F64], Bool));
        #[cfg(feature = "dynload")]
        m.add_str("load", load, Dfn::nl(vec![Str], Type::result()));
        #[cfg(feature = "dynload")]
        m.add_str(
            "load__source_imports",
            load__source_imports,
            Dfn::nl(vec![Str, Type::array()], Type::result()),
        );
        m.add_str(
            "module__in_string_imports",
            module__in_string_imports,
            Dfn::nl(vec![Str, Str, Type::array()], Type::result()),
        );
        m.add_str(
            "check__in_string_imports",
            check__in_string_imports,
            Dfn::nl(
                vec![Str, Str, Type::array()],
                Type::Result(Box::new(Type::Array(Box::new(Type::Object)))),
            ),
        );
        m.add_str("call", _call, Dfn::nl(vec![Any, Str, Type::array()], Void));
        m.add_str(
            "call_ret",
            call_ret,
            Dfn::nl(vec![Any, Str, Type::array()], Any),
        );
        m.add_str("functions", functions, Dfn::nl(vec![], Any));
        m.add_str(
            "functions__module",
            functions__module,
            Dfn::nl(vec![Any], Any),
        );
        m.add_str("is_err", is_err, Dfn::nl(vec![Type::result()], Bool));
        m.add_str("is_ok", is_ok, Dfn::nl(vec![Type::result()], Bool));
        m.add_str("min", min, Dfn::nl(vec![Type::Array(Box::new(F64))], F64));
        m.add_str("max", max, Dfn::nl(vec![Type::Array(Box::new(F64))], F64));
        m.add_str("unwrap", unwrap, Dfn::nl(vec![Any], Any));
        m.add_str(
            "why",
            why,
            Dfn::nl(vec![Type::Secret(Box::new(Bool))], Type::array()),
        );
        m.add_str(
            "where",
            _where,
            Dfn::nl(vec![Type::Secret(Box::new(F64))], Type::array()),
        );
        m.add_str(
            "explain_why",
            explain_why,
            Dfn::nl(vec![Bool, Any], Type::Secret(Box::new(Bool))),
        );
        m.add_str(
            "explain_where",
            explain_where,
            Dfn::nl(vec![F64, Any], Type::Secret(Box::new(F64))),
        );
        m.add_str("head", head, Dfn::nl(vec![Link], Any));
        m.add_str("tip", tip, Dfn::nl(vec![Link], Type::Option(Box::new(Any))));
        m.add_str("tail", tail, Dfn::nl(vec![Link], Link));
        m.add_str("neck", neck, Dfn::nl(vec![Link], Link));
        m.add_str("is_empty", is_empty, Dfn::nl(vec![Link], Bool));
        m.add_unop_str("len", len, Dfn::nl(vec![Type::array()], F64));
        m.add_str(
            "push_ref(mut,_)",
            push_ref,
            Dfn {
                lts: vec![Lt::Default, Lt::Arg(0)],
                tys: vec![Type::array(), Any],
                ret: Void,
                ext: vec![],
                lazy: LAZY_NO,
            },
        );
        m.add_str(
            "insert_ref(mut,_,_)",
            insert_ref,
            Dfn {
                lts: vec![Lt::Default, Lt::Default, Lt::Arg(0)],
                tys: vec![Type::array(), F64, Any],
                ret: Void,
                ext: vec![],
                lazy: LAZY_NO,
            },
        );
        m.add_str("push(mut,_)", push, Dfn::nl(vec![Type::array(), Any], Void));
        m.add_str(
            "insert(mut,_,_)",
            insert,
            Dfn {
                lts: vec![Lt::Default; 3],
                tys: vec![Type::array(), F64, Any],
                ret: Void,
                ext: vec![],
                lazy: LAZY_NO,
            },
        );
        m.add_str(
            "pop(mut)",
            pop,
            Dfn {
                lts: vec![Lt::Return],
                tys: vec![Type::array()],
                ret: Any,
                ext: vec![],
                lazy: LAZY_NO,
            },
        );
        m.add_str(
            "remove(mut,_)",
            remove,
            Dfn {
                lts: vec![Lt::Return, Lt::Default],
                tys: vec![Type::array(), F64],
                ret: Any,
                ext: vec![],
                lazy: LAZY_NO,
            },
        );
        m.add_str("reverse(mut)", reverse, Dfn::nl(vec![Type::array()], Void));
        m.add_str("clear(mut)", clear, Dfn::nl(vec![Type::array()], Void));
        m.add_str(
            "swap(mut,_,_)",
            swap,
            Dfn::nl(vec![Type::array(), F64, F64], Void),
        );
        m.add_str(
            "unwrap_or",
            unwrap_or,
            Dfn {
                lts: vec![Lt::Default; 2],
                tys: vec![Any; 2],
                ret: Any,
                ext: vec![],
                lazy: LAZY_UNWRAP_OR,
            },
        );
        m.add_str("unwrap_err", unwrap_err, Dfn::nl(vec![Any], Any));
        m.add_str(
            "meta__syntax_in_string",
            meta__syntax_in_string,
            Dfn::nl(
                vec![Any, Str, Str],
                Type::Result(Box::new(Type::Array(Box::new(Type::array())))),
            ),
        );
        m.add_str(
            "save__data_file",
            save__data_file,
            Dfn::nl(vec![Any, Str], Str),
        );
        m.add_str(
            "json_from_meta_data",
            json_from_meta_data,
            Dfn::nl(vec![Type::Array(Box::new(Type::array()))], Str),
        );
        m.add_str(
            "errstr__string_start_len_msg",
            errstr__string_start_len_msg,
            Dfn::nl(vec![Str, F64, F64, Str], Str),
        );
        m.add_str("has", has, Dfn::nl(vec![Object, Str], Bool));
        m.add_str(
            "keys",
            keys,
            Dfn::nl(vec![Object], Type::Array(Box::new(Str))),
        );
        m.add_str(
            "chars",
            chars,
            Dfn::nl(vec![Str], Type::Array(Box::new(Str))),
        );
        #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
        m.add_str("wait_next", wait_next, Dfn::nl(vec![Type::in_ty()], Any));
        #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
        m.add_str("next", next, Dfn::nl(vec![Type::in_ty()], Type::option()));

        m.no_ns();
        m
    }

    /// Sets namespace for following added functions.
    pub fn ns(&mut self, ns: &str) {
        self.register_namespace = Arc::new(ns.split("::").map(|s| Arc::new(s.into())).collect());
    }

    /// Sets no namespace.
    pub fn no_ns(&mut self) {
        self.register_namespace = Arc::new(vec![]);
    }

    pub(crate) fn register(&mut self, function: ast::Function) {
        self.functions.push(function);
    }

    /// Find function relative another function index.
    pub fn find_function(&self, name: &Arc<String>, relative: usize) -> FnIndex {
        for (i, f) in self.functions.iter().enumerate().rev() {
            if &f.name == name {
                return FnIndex::Loaded(i as isize - relative as isize);
            }
        }
        for f in self.ext_prelude.iter().rev() {
            if &f.name == name {
                return match f.f {
                    FnExt::Return(ff) => {
                        if f.p.lazy == LAZY_NO {
                            FnIndex::Return(FnReturnRef(ff))
                        } else {
                            FnIndex::Lazy(FnReturnRef(ff), f.p.lazy)
                        }
                    }
                    FnExt::BinOp(ff) => FnIndex::BinOp(FnBinOpRef(ff)),
                    FnExt::UnOp(ff) => FnIndex::UnOp(FnUnOpRef(ff)),
                    FnExt::Void(ff) => FnIndex::Void(FnVoidRef(ff)),
                };
            }
        }
        FnIndex::None
    }

    /// Generates an error message.
    pub(crate) fn error(&self, range: Range, msg: &str, rt: &Runtime) -> String {
        let fnindex = if let Some(x) = rt.call_stack.last() {
            x.index
        } else {
            return msg.into();
        };
        self.error_fnindex(range, msg, fnindex)
    }

    /// Generates an error with a function index.
    pub(crate) fn error_fnindex(&self, range: Range, msg: &str, fnindex: usize) -> String {
        let source = &self.functions[fnindex].source;
        self.error_source(range, msg, source)
    }

    /// Generates an error message with a source.
    pub(crate) fn error_source(&self, range: Range, msg: &str, source: &Arc<String>) -> String {
        use piston_meta::ParseErrorHandler;

        let mut w: Vec<u8> = vec![];
        ParseErrorHandler::new(source)
            .write_msg(&mut w, range, msg)
            .unwrap();
        String::from_utf8(w).unwrap()
    }

    /// Adds a new external prelude function.
    pub fn add<T>(&mut self, name: Arc<String>, f: fn(&mut Runtime) -> T, prelude_function: Dfn)
    where
        fn(&mut Runtime) -> T: Into<FnExt>,
    {
        self.ext_prelude.push(FnExternal {
            namespace: self.register_namespace.clone(),
            name,
            f: f.into(),
            p: prelude_function,
        });
    }

    /// Adds a new external prelude function.
    pub fn add_str<T>(&mut self, name: &str, f: fn(&mut Runtime) -> T, prelude_function: Dfn)
    where
        fn(&mut Runtime) -> T: Into<FnExt>,
    {
        self.ext_prelude.push(FnExternal {
            namespace: self.register_namespace.clone(),
            name: Arc::new(name.into()),
            f: f.into(),
            p: prelude_function,
        });
    }

    /// Adds a new external prelude binary operator.
    pub fn add_binop(
        &mut self,
        name: Arc<String>,
        f: fn(&Variable, &Variable) -> Result<Variable, String>,
        prelude_function: Dfn,
    ) {
        self.ext_prelude.push(FnExternal {
            namespace: self.register_namespace.clone(),
            name,
            f: f.into(),
            p: prelude_function,
        });
    }

    /// Adds a new external prelude unary operator.
    pub fn add_unop(
        &mut self,
        name: Arc<String>,
        f: fn(&Variable) -> Result<Variable, String>,
        prelude_function: Dfn,
    ) {
        self.ext_prelude.push(FnExternal {
            namespace: self.register_namespace.clone(),
            name,
            f: f.into(),
            p: prelude_function,
        });
    }

    /// Adds a new external prelude unary operator.
    pub fn add_unop_str(
        &mut self,
        name: &str,
        f: fn(&Variable) -> Result<Variable, String>,
        prelude_function: Dfn,
    ) {
        self.add_unop(Arc::new(name.into()), f, prelude_function)
    }
}
