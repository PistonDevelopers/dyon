#![cfg_attr(test, feature(test))]
extern crate piston_meta;
extern crate rand;
extern crate range;
extern crate read_color;
extern crate read_token;
#[cfg(feature = "http")]
extern crate reqwest;
#[macro_use]
extern crate lazy_static;
extern crate vecmath;

use std::any::Any;
use std::fmt;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use range::Range;
use piston_meta::MetaData;

pub mod ast;
pub mod runtime;
mod lifetime;
mod intrinsics;
mod prelude;
pub mod embed;
mod ty;
mod link;
pub mod macros;
mod vec4;
mod mat4;
mod write;

mod grab;
mod dyon_std;

pub use runtime::Runtime;
pub use prelude::{Lt, Prelude, Dfn};
pub use ty::Type;
pub use link::Link;
pub use vec4::Vec4;
pub use mat4::Mat4;

/// A common error message when there is no value on the stack.
pub const TINVOTS: &'static str = "There is no value on the stack";

/// Type alias for Dyon arrays.
pub type Array = Arc<Vec<Variable>>;
/// Type alias for Dyon objects.
pub type Object = Arc<HashMap<Arc<String>, Variable>>;
/// Type alias for Rust objects.
pub type RustObject = Arc<Mutex<Any>>;

/// Stores Dyon errors.
#[derive(Debug, Clone)]
pub struct Error {
    /// The error message.
    pub message: Variable,
    /// Extra information to help debug error.
    /// Stores error messages for all `?` operators.
    pub trace: Vec<String>,
}

/// Stores a thread handle.
#[derive(Clone)]
pub struct Thread {
    /// The handle of the thread.
    pub handle: Option<Arc<Mutex<JoinHandle<Result<Variable, String>>>>>,
}

impl Thread {
    /// Creates a new thread handle.
    pub fn new(handle: JoinHandle<Result<Variable, String>>) -> Thread {
        Thread {
            handle: Some(Arc::new(Mutex::new(handle)))
        }
    }

    /// Removes the thread handle from the stack.
    /// This is to prevent an extra reference when resolving the variable.
    pub fn invalidate_handle(
        rt: &mut Runtime,
        var: Variable
    ) -> Result<JoinHandle<Result<Variable, String>>, String> {
        use std::error::Error;

        let thread = match var {
            Variable::Ref(ind) => {
                use std::mem::replace;

                match replace(&mut rt.stack[ind], Variable::Thread(Thread { handle: None })) {
                    Variable::Thread(th) => th,
                    x => return Err(rt.expected(&x, "Thread"))
                }
            }
            Variable::Thread(thread) => thread,
            x => return Err(rt.expected(&x, "Thread"))
        };
        let handle = match thread.handle {
            None => return Err("The Thread has already been invalidated".into()),
            Some(thread) => thread
        };
        let mutex = try!(Arc::try_unwrap(handle).map_err(|_|
            format!("{}\nCan not access Thread because there is \
            more than one reference to it", rt.stack_trace())));
        mutex.into_inner().map_err(|err|
            format!("{}\nCan not lock Thread mutex:\n{}", rt.stack_trace(), err.description()))
    }
}

impl fmt::Debug for Thread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "thread")
    }
}

/// Prevents unsafe references from being accessed outside library.
#[derive(Debug, Clone)]
pub struct UnsafeRef(*mut Variable);

/// Stores closure environment.
#[derive(Clone)]
pub struct ClosureEnvironment {
    /// The module that the closure was created.
    pub module: Arc<Module>,
    /// Relative index, used to resolve function indices.
    pub relative: usize,
}

impl fmt::Debug for ClosureEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ClosureEnvironment")
    }
}

/// Dyon variable.
#[derive(Debug, Clone)]
pub enum Variable {
    /// Reference.
    Ref(usize),
    /// Return handle.
    Return,
    /// Boolean.
    Bool(bool, Option<Box<Vec<Variable>>>),
    /// F64.
    F64(f64, Option<Box<Vec<Variable>>>),
    /// 4D vector.
    Vec4([f32; 4]),
    /// 4D matrix.
    Mat4(Box<[[f32; 4]; 4]>),
    /// Text.
    Text(Arc<String>),
    /// Array.
    Array(Array),
    /// Object.
    Object(Object),
    /// Link.
    Link(Box<Link>),
    /// Unsafe reference.
    UnsafeRef(UnsafeRef),
    /// Rust object.
    RustObject(RustObject),
    /// Option.
    Option(Option<Box<Variable>>),
    /// Result.
    Result(Result<Box<Variable>, Box<Error>>),
    /// Thread handle.
    Thread(Thread),
    /// Stores closure together with a closure environment,
    /// which makes sure that the closure can be called correctly
    /// no matter where it goes.
    Closure(Arc<ast::Closure>, Box<ClosureEnvironment>),
    /// In-type.
    In(Arc<Mutex<::std::sync::mpsc::Receiver<Variable>>>),
}

/// This is requires because `UnsafeRef(*mut Variable)` can not be sent across threads.
/// The lack of `UnsafeRef` variant when sending across threads is guaranteed at language level.
/// The interior of `UnsafeRef` can not be accessed outside this library.
unsafe impl Send for Variable {}

impl Variable {
    /// Creates a variable of type `f64`.
    pub fn f64(val: f64) -> Variable {
        Variable::F64(val, None)
    }

    /// Creates a variable of type `bool`.
    pub fn bool(val: bool) -> Variable {
        Variable::Bool(val, None)
    }

    /// Returns type of variable.
    pub fn typeof_var(&self) -> Arc<String> {
        use self::runtime::*;

        match self {
            &Variable::Text(_) => text_type.clone(),
            &Variable::F64(_, _) => f64_type.clone(),
            &Variable::Vec4(_) => vec4_type.clone(),
            &Variable::Mat4(_) => mat4_type.clone(),
            &Variable::Return => return_type.clone(),
            &Variable::Bool(_, _) => bool_type.clone(),
            &Variable::Object(_) => object_type.clone(),
            &Variable::Array(_) => array_type.clone(),
            &Variable::Link(_) => link_type.clone(),
            &Variable::Ref(_) => ref_type.clone(),
            &Variable::UnsafeRef(_) => unsafe_ref_type.clone(),
            &Variable::RustObject(_) => rust_object_type.clone(),
            &Variable::Option(_) => option_type.clone(),
            &Variable::Result(_) => result_type.clone(),
            &Variable::Thread(_) => thread_type.clone(),
            &Variable::Closure(_, _) => closure_type.clone(),
            &Variable::In(_) => in_type.clone(),
        }
    }

    fn deep_clone(&self, stack: &Vec<Variable>) -> Variable {
        use Variable::*;

        match *self {
            F64(_, _) => self.clone(),
            Vec4(_) => self.clone(),
            Mat4(_) => self.clone(),
            Return => self.clone(),
            Bool(_, _) => self.clone(),
            Text(_) => self.clone(),
            Object(ref obj) => {
                let mut res = obj.clone();
                for (_, val) in Arc::make_mut(&mut res) {
                    *val = val.deep_clone(stack);
                }
                Object(res)
            }
            Array(ref arr) => {
                let mut res = arr.clone();
                for it in Arc::make_mut(&mut res) {
                    *it = it.deep_clone(stack);
                }
                Array(res)
            }
            Link(_) => self.clone(),
            Ref(ind) => {
                stack[ind].deep_clone(stack)
            }
            UnsafeRef(_) => panic!("Unsafe reference can not be cloned"),
            RustObject(_) => self.clone(),
            Option(None) => Variable::Option(None),
            // `some(x)` always uses deep clone, so it does not contain references.
            Option(Some(ref v)) => Option(Some(v.clone())),
            // `ok(x)` always uses deep clone, so it does not contain references.
            Result(Ok(ref ok)) => Result(Ok(ok.clone())),
            // `err(x)` always uses deep clone, so it does not contain references.
            Result(Err(ref err)) => Result(Err(err.clone())),
            Thread(_) => self.clone(),
            Closure(_, _) => self.clone(),
            In(_) => self.clone(),
        }
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Variable) -> bool {
        match (self, other) {
            (&Variable::Return, _) => false,
            (&Variable::Bool(a, _), &Variable::Bool(b, _)) => a == b,
            (&Variable::F64(a, _), &Variable::F64(b, _)) => a == b,
            (&Variable::Text(ref a), &Variable::Text(ref b)) => a == b,
            (&Variable::Object(ref a), &Variable::Object(ref b)) => a == b,
            (&Variable::Array(ref a), &Variable::Array(ref b)) => a == b,
            (&Variable::Ref(_), _) => false,
            (&Variable::UnsafeRef(_), _) => false,
            (&Variable::RustObject(_), _) => false,
            _ => false,
        }
    }
}

/// Refers to a function.
#[derive(Clone, Copy, Debug)]
pub enum FnIndex {
    /// No function.
    None,
    /// An intrinsic function.
    Intrinsic(usize),
    /// Relative to function you call from.
    Loaded(isize),
    /// External function with no return value.
    ExternalVoid(FnExternalRef),
    /// Extern function with return value.
    ExternalReturn(FnExternalRef),
}

/// Used to store direct reference to external function.
#[derive(Copy)]
pub struct FnExternalRef(pub fn(&mut Runtime) -> Result<(), String>);

impl Clone for FnExternalRef {
    fn clone(&self) -> FnExternalRef {
        *self
    }
}

impl fmt::Debug for FnExternalRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FnExternalRef")
    }
}

struct FnExternal {
    namespace: Arc<Vec<Arc<String>>>,
    name: Arc<String>,
    f: fn(&mut Runtime) -> Result<(), String>,
    p: Dfn,
}

impl Clone for FnExternal {
    fn clone(&self) -> FnExternal {
        FnExternal {
            namespace: self.namespace.clone(),
            name: self.name.clone(),
            f: self.f,
            p: self.p.clone(),
        }
    }
}

/// Stores functions for a Dyon module.
#[derive(Clone)]
pub struct Module {
    functions: Vec<ast::Function>,
    ext_prelude: Vec<FnExternal>,
    intrinsics: Arc<HashMap<Arc<String>, usize>>,
    register_namespace: Arc<Vec<Arc<String>>>,
}

impl Module {
    /// Creates a new module with standard library.
    pub fn new() -> Module {
        use Type::*;
        use dyon_std::*;

        let mut m = Module::new_intrinsics(Arc::new(Prelude::new_intrinsics().functions));
        m.add_str("x", x, Dfn::nl(vec![Vec4], F64));
        m.add_str("y", y, Dfn::nl(vec![Vec4], F64));
        m.add_str("z", z, Dfn::nl(vec![Vec4], F64));
        m.add_str("w", w, Dfn::nl(vec![Vec4], F64));
        m.add_str("det", det, Dfn::nl(vec![Mat4], F64));
        m.add_str("inv", inv, Dfn::nl(vec![Mat4], Mat4));
        m.add_str("mov", mov, Dfn::nl(vec![Vec4], Mat4));
        m.add_str("rot__axis_angle", rot__axis_angle, Dfn::nl(vec![Vec4, F64], Mat4));
        m.add_str("ortho__pos_right_up_forward", ortho__pos_right_up_forward,
                  Dfn::nl(vec![Vec4; 4], Mat4));
        m.add_str("proj__fov_near_far_ar", proj__fov_near_far_ar, Dfn::nl(vec![F64; 4], Mat4));
        m.add_str("mvp__model_view_projection", mvp__model_view_projection,
                  Dfn::nl(vec![Mat4; 3], Mat4));
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
        m.add_str("println", println, Dfn::nl(vec![Any], Void));
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
        m.add_str("sleep", sleep, Dfn::nl(vec![F64], Void));
        m.add_str("random", random, Dfn::nl(vec![], F64));
        m.add_str("tau", tau, Dfn::nl(vec![], F64));
        m.add_str("read_line", read_line, Dfn::nl(vec![], Text));
        m.add_str("read_number", read_number, Dfn::nl(vec![Text], F64));
        m.add_str("parse_number", parse_number, Dfn::nl(vec![Text], Option(Box::new(Type::F64))));
        m.add_str("trim", trim, Dfn::nl(vec![Text], Text));
        m.add_str("trim_left", trim_left, Dfn::nl(vec![Text], Text));
        m.add_str("trim_right", trim_right, Dfn::nl(vec![Text], Text));
        m.add_str("str", _str, Dfn::nl(vec![Any], Text));
        m.add_str("json_string", json_string, Dfn::nl(vec![Text], Text));
        m.add_str("str__color", str__color, Dfn::nl(vec![Vec4], Text));
        m.add_str("srgb_to_linear__color", srgb_to_linear__color, Dfn::nl(vec![Vec4], Vec4));
        m.add_str("linear_to_srgb__color", linear_to_srgb__color, Dfn::nl(vec![Vec4], Vec4));
        m.add_str("typeof", _typeof, Dfn::nl(vec![Any], Text));
        m.add_str("debug", debug, Dfn::nl(vec![], Void));
        m.add_str("backtrace", backtrace, Dfn::nl(vec![], Void));
        m.add_str("none", none, Dfn::nl(vec![], Type::option()));
        m.add_str("some", some, Dfn::nl(vec![Any], Type::option()));
        m.add_str("ok", ok, Dfn::nl(vec![Any], Type::result()));
        m.add_str("err", err, Dfn::nl(vec![Any], Type::result()));
        m.add_str("dir__angle", dir__angle, Dfn::nl(vec![F64], Vec4));
        m.add_str("load__meta_file", load__meta_file, Dfn::nl(vec![Type::Text; 2],
            Type::Result(Box::new(Type::Array(Box::new(Type::array()))))
        ));
        m.add_str("load__meta_url", load__meta_url, Dfn::nl(vec![Type::Text; 2],
            Type::Result(Box::new(Type::Array(Box::new(Type::array()))))
        ));
        m.add_str("syntax__in_string", syntax__in_string,
                  Dfn::nl(vec![Type::Text; 2], Type::Result(Box::new(Type::Any))));
        m.add_str("download__url_file", download__url_file,
                  Dfn::nl(vec![Type::Text; 2], Type::Result(Box::new(Type::Text))));
        m.add_str("save__string_file", save__string_file,
                  Dfn::nl(vec![Type::Text; 2], Type::Result(Box::new(Type::Text))));
        m.add_str("load_string__file", load_string__file,
                  Dfn::nl(vec![Text], Type::Result(Box::new(Type::Text))));
        m.add_str("load_string__url", load_string__url,
                  Dfn::nl(vec![Text], Type::Result(Box::new(Type::Text))));
        m.add_str("join__thread", join__thread,
                  Dfn::nl(vec![Type::thread()], Type::Result(Box::new(Type::Any))));
        m.add_str("load_data__file", load_data__file,
                  Dfn::nl(vec![Text], Type::Result(Box::new(Type::Any))));
        m.add_str("load_data__string", load_data__string,
                  Dfn::nl(vec![Text], Type::Result(Box::new(Type::Any))));
        m.add_str("args_os", args_os, Dfn::nl(vec![], Type::Array(Box::new(Type::Text))));
        m.add_str("now", now, Dfn::nl(vec![], F64));
        m.add_str("is_nan", is_nan, Dfn::nl(vec![F64], Bool));
        m
    }

    /// Creates a new module with custom intrinsics.
    pub fn new_intrinsics(intrinsics: Arc<HashMap<Arc<String>, usize>>) -> Module {
        Module {
            functions: vec![],
            ext_prelude: vec![],
            intrinsics: intrinsics,
            register_namespace: Arc::new(vec![]),
        }
    }

    /// Sets namespace for following added functions.
    pub fn ns(&mut self, ns: &str) {
        self.register_namespace = Arc::new(ns
            .split("::")
            .map(|s| Arc::new(s.into()))
            .collect());
    }

    /// Sets no namespace.
    pub fn no_ns(&mut self) {
        self.register_namespace = Arc::new(vec![]);
    }

    fn register(&mut self, function: ast::Function) {
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
                return if f.p.returns() {
                    FnIndex::ExternalReturn(FnExternalRef(f.f))
                } else {
                    FnIndex::ExternalVoid(FnExternalRef(f.f))
                };
            }
        }
        match self.intrinsics.get(name) {
            None => FnIndex::None,
            Some(&ind) => FnIndex::Intrinsic(ind)
        }
    }

    /// Generates an error message.
    fn error(&self, range: Range, msg: &str, rt: &Runtime) -> String {
        let fnindex = if let Some(x) = rt.call_stack.last() {x.index}
                      else {return msg.into()};
        self.error_fnindex(range, msg, fnindex)
    }

    /// Generates an error with a function index.
    fn error_fnindex(&self, range: Range, msg: &str, fnindex: usize) -> String {
        let source = &self.functions[fnindex].source;
        self.error_source(range, msg, source)
    }

    /// Generates an error message with a source.
    fn error_source(&self, range: Range, msg: &str, source: &Arc<String>) -> String {
        use piston_meta::ParseErrorHandler;

        let mut w: Vec<u8> = vec![];
        ParseErrorHandler::new(source)
            .write_msg(&mut w, range, &format!("{}", msg))
            .unwrap();
        String::from_utf8(w).unwrap()
    }

    /// Adds a new external prelude function.
    pub fn add(
        &mut self,
        name: Arc<String>,
        f: fn(&mut Runtime) -> Result<(), String>,
        prelude_function: Dfn
    ) {
        self.ext_prelude.push(FnExternal {
            namespace: self.register_namespace.clone(),
            name: name.clone(),
            f: f,
            p: prelude_function,
        });
    }

    /// Adds a new external prelude function.
    pub fn add_str(
        &mut self,
        name: &str,
        f: fn(&mut Runtime) -> Result<(), String>,
        prelude_function: Dfn
    ) {
        self.ext_prelude.push(FnExternal {
            namespace: self.register_namespace.clone(),
            name: Arc::new(name.into()),
            f: f,
            p: prelude_function,
        });
    }
}

/// Runs a program using a source file.
pub fn run(source: &str) -> Result<(), String> {
    let mut module = Module::new();
    try!(load(source, &mut module));
    let mut runtime = runtime::Runtime::new();
    try!(runtime.run(&Arc::new(module)));
    Ok(())
}

/// Runs a program from a string.
pub fn run_str(source: &str, d: Arc<String>) -> Result<(), String> {
    let mut module = Module::new();
    try!(load_str(source, d, &mut module));
    let mut runtime = runtime::Runtime::new();
    try!(runtime.run(&Arc::new(module)));
    Ok(())
}

/// Used to call specific functions with arguments.
pub struct Call {
    args: Vec<Variable>,
    name: Arc<String>,
}

impl Call {
    /// Creates a new call.
    pub fn new(name: &str) -> Call {
        Call {
            args: vec![],
            name: Arc::new(name.into())
        }
    }

    /// Push value to argument list.
    pub fn arg<T: embed::PushVariable>(mut self, val: T) -> Self {
        self.args.push(val.push_var());
        self
    }

    /// Push Vec4 to argument list.
    pub fn vec4<T: embed::ConvertVec4>(mut self, val: T) -> Self {
        self.args.push(Variable::Vec4(val.to()));
        self
    }

    /// Push Rust object to argument list.
    pub fn rust<T: 'static>(mut self, val: T) -> Self {
        self.args.push(Variable::RustObject(Arc::new(Mutex::new(val)) as RustObject));
        self
    }

    /// Run call without any return value.
    pub fn run(&self, runtime: &mut Runtime, module: &Arc<Module>) -> Result<(), String> {
        runtime.call_str(&self.name, &self.args, module)
    }

    /// Run call with return value.
    pub fn run_ret<T: embed::PopVariable>(&self, runtime: &mut Runtime, module: &Arc<Module>) -> Result<T, String> {
        let val = runtime.call_str_ret(&self.name, &self.args, module)?;
        T::pop_var(runtime, runtime.resolve(&val))
    }

    /// Convert return value to a Vec4 convertible type.
    pub fn run_vec4<T: embed::ConvertVec4>(&self, runtime: &mut Runtime, module: &Arc<Module>) -> Result<T, String> {
        let val = runtime.call_str_ret(&self.name, &self.args, module)?;
        match runtime.resolve(&val) {
            &Variable::Vec4(val) => Ok(T::from(val)),
            x => Err(runtime.expected(x, "vec4"))
        }
    }
}

/// Loads source from file.
pub fn load(source: &str, module: &mut Module) -> Result<(), String> {
    use std::fs::File;
    use std::io::Read;

    let mut data_file = try!(File::open(source).map_err(|err|
        format!("Could not open `{}`, {}", source, err)));
    let mut data = Arc::new(String::new());
    data_file.read_to_string(Arc::make_mut(&mut data)).unwrap();
    load_str(source, data, module)
}

/// Loads a source from string.
///
/// - source - The name of source file
/// - d - The data of source file
/// - module - The module to load the source
pub fn load_str(source: &str, d: Arc<String>, module: &mut Module) -> Result<(), String> {
    use std::thread;
    use piston_meta::{parse_errstr, syntax_errstr, Syntax};

    lazy_static! {
        static ref SYNTAX_RULES: Result<Syntax, String> = {
            let syntax = include_str!("../assets/syntax.txt");
            syntax_errstr(syntax)
        };
    }

    let syntax_rules = try!(SYNTAX_RULES.as_ref()
        .map_err(|err| err.clone()));

    let mut data = vec![];
    try!(parse_errstr(syntax_rules, &d, &mut data).map_err(
        |err| format!("In `{}:`\n{}", source, err)
    ));

    let check_data = data.clone();
    let prelude = Arc::new(Prelude::from_module(module));

    // Do lifetime checking in parallel directly on meta data.
    let handle = thread::spawn(move || {
        let check_data = check_data;
        lifetime::check(&check_data, &prelude)
    });

    // Convert to AST.
    let mut ignored = vec![];
    let conv_res = ast::convert(Arc::new(source.into()), d.clone(), &data, &mut ignored, module);

    // Check that lifetime checking succeeded.
    match handle.join().unwrap() {
        Ok(refined_rets) => {
            for (name, ty) in &refined_rets {
                if let FnIndex::Loaded(f_index) = module.find_function(name, 0) {
                    let f = &mut module.functions[f_index as usize];
                    f.ret = ty.clone();
                }
            }
        }
        Err(err_msg) => {
            use std::io::Write;
            use piston_meta::ParseErrorHandler;

            let (range, msg) = err_msg.decouple();

            let mut buf: Vec<u8> = vec![];
            writeln!(&mut buf, "In `{}`:\n", source).unwrap();
            ParseErrorHandler::new(&d)
                .write_msg(&mut buf, range, &msg)
                .unwrap();
            return Err(String::from_utf8(buf).unwrap())
        }
    }

    check_ignored_meta_data(&conv_res, source, &d, &data, &ignored)
}

/// Loads a source from meta data.
/// Assumes the source passes the lifetime checker.
pub fn load_meta(
    source: &str,
    d: Arc<String>,
    data: &[Range<MetaData>],
    module: &mut Module
) -> Result<(), String> {
    // Convert to AST.
    let mut ignored = vec![];
    let conv_res = ast::convert(Arc::new(source.into()), d.clone(), &data, &mut ignored, module);

    check_ignored_meta_data(&conv_res, source, &d, data, &ignored)
}

fn check_ignored_meta_data(
    conv_res: &Result<(), ()>,
    source: &str,
    d: &Arc<String>,
    data: &[Range<MetaData>],
    ignored: &[Range],
) -> Result<(), String> {
    use piston_meta::json;

    if ignored.len() > 0 || conv_res.is_err() {
        use std::io::Write;
        use piston_meta::ParseErrorHandler;

        let mut buf: Vec<u8> = vec![];
        if ignored.len() > 0 {
            writeln!(&mut buf, "Some meta data was ignored in the syntax").unwrap();
            writeln!(&mut buf, "START IGNORED").unwrap();
            json::write(&mut buf, &data[ignored[0].iter()]).unwrap();
            writeln!(&mut buf, "END IGNORED").unwrap();

            writeln!(&mut buf, "In `{}`:\n", source).unwrap();
            ParseErrorHandler::new(&d)
                .write_msg(&mut buf,
                           data[ignored[0].iter()][0].range(),
                           "Could not understand this")
                .unwrap();
        }
        if let &Err(()) = conv_res {
            writeln!(&mut buf, "Conversion error").unwrap();
        }
        return Err(String::from_utf8(buf).unwrap());
    }

    Ok(())
}

/// Reports and error to standard output.
pub fn error(res: Result<(), String>) -> bool {
    match res {
        Err(err) => {
            println!("");
            println!(" --- ERROR --- ");
            println!("{}", err);
            true
        }
        Ok(()) => false
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::run;
    use self::test::Bencher;

    #[test]
    fn variable_size() {
        use std::mem::size_of;
        use std::sync::Arc;
        use super::*;

        /*
        Ref(usize),
        Return,
        Bool(bool, Option<Box<Vec<Variable>>>),
        F64(f64, Option<Box<Vec<Variable>>>),
        Vec4([f32; 4]),
        Text(Arc<String>),
        Array(Array),
        Object(Object),
        Link(Box<Link>),
        UnsafeRef(UnsafeRef),
        RustObject(RustObject),
        Option(Option<Box<Variable>>),
        Result(Result<Box<Variable>, Box<Error>>),
        Thread(Thread),
        */

        println!("Link {}", size_of::<Box<Link>>());
        println!("[f32; 4] {}", size_of::<[f32; 4]>());
        println!("Result {}", size_of::<Result<Box<Variable>, Box<Error>>>());
        println!("Thread {}", size_of::<Thread>());
        println!("Secret {}", size_of::<Option<Box<Vec<Variable>>>>());
        println!("Text {}", size_of::<Arc<String>>());
        println!("Array {}", size_of::<Array>());
        println!("Object {}", size_of::<Object>());
        assert_eq!(size_of::<Variable>(), 24);
    }

    fn run_bench(source: &str) {
        run(source).unwrap_or_else(|err| panic!("{}", err));
    }

    #[bench]
    fn bench_add(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/add.dyon"));
    }

    #[bench]
    fn bench_add_n(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/add_n.dyon"));
    }

    #[bench]
    fn bench_sum(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/sum.dyon"));
    }

    #[bench]
    fn bench_main(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/main.dyon"));
    }

    #[bench]
    fn bench_array(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/array.dyon"));
    }

    #[bench]
    fn bench_object(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/object.dyon"));
    }

    #[bench]
    fn bench_call(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/call.dyon"));
    }

    #[bench]
    fn bench_n_body(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/n_body.dyon"));
    }

    #[bench]
    fn bench_len(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/len.dyon"));
    }

    #[bench]
    fn bench_min_fn(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/min_fn.dyon"));
    }

    #[bench]
    fn bench_min(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/min.dyon"));
    }

    #[bench]
    fn bench_primes(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/primes.dyon"));
    }

    #[bench]
    fn bench_primes_trad(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/primes_trad.dyon"));
    }

    #[bench]
    fn bench_threads_no_go(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/threads_no_go.dyon"));
    }

    #[bench]
    fn bench_threads_go(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/threads_go.dyon"));
    }

    #[bench]
    fn bench_push_array(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/push_array.dyon"));
    }

    #[bench]
    fn bench_push_link(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/push_link.dyon"));
    }

    #[bench]
    fn bench_push_link_for(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/push_link_for.dyon"));
    }

    #[bench]
    fn bench_push_link_go(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/push_link_go.dyon"));
    }

    #[bench]
    fn bench_push_str(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/push_str.dyon"));
    }

    #[bench]
    fn bench_push_in(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/push_in.dyon"));
    }
}
