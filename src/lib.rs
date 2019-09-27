//! # Dyon - a rusty dynamically typed scripting language
//!
//! [Tutorial](http://www.piston.rs/dyon-tutorial/)
//!
//! If you want to say thanks for Dyon, or you found any of these ideas inspiring,
//! please donate a small amount of money as a symbolic gesture to PayPal `post at cutoutpro.com`.
//! Write a sentence describing why you think this is a good language and what you want to achieve.
//! I (bvssvni, creator of Dyon) appreciate grateful and heart-warming responses!

#![cfg_attr(test, feature(test))]
#![deny(missing_docs)]
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
extern crate tree_mem_sort;

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
mod prelude;
pub mod embed;
mod ty;
mod link;
pub mod macros;
mod vec4;
mod mat4;
mod write;
mod module;

mod grab;
mod dyon_std;

pub use runtime::Runtime;
pub use prelude::{Lt, Prelude, Dfn};
pub use ty::Type;
pub use link::Link;
pub use vec4::Vec4;
pub use mat4::Mat4;
pub use ast::Lazy;
pub use module::Module;

/// A common error message when there is no value on the stack.
pub const TINVOTS: &str = "There is no value on the stack";

lazy_static!{
    pub(crate) static ref LESS: Arc<String> = Arc::new("less".into());
    pub(crate) static ref LESS_OR_EQUAL: Arc<String> = Arc::new("less_or_equal".into());
    pub(crate) static ref GREATER: Arc<String> = Arc::new("greater".into());
    pub(crate) static ref GREATER_OR_EQUAL: Arc<String> = Arc::new("greater_or_equal".into());
    pub(crate) static ref EQUAL: Arc<String> = Arc::new("equal".into());
    pub(crate) static ref NOT_EQUAL: Arc<String> = Arc::new("not_equal".into());
    pub(crate) static ref AND_ALSO: Arc<String> = Arc::new("and_also".into());
    pub(crate) static ref OR_ELSE: Arc<String> = Arc::new("or_else".into());
    pub(crate) static ref ADD: Arc<String> = Arc::new("add".into());
    pub(crate) static ref SUB: Arc<String> = Arc::new("sub".into());
    pub(crate) static ref MUL: Arc<String> = Arc::new("mul".into());
    pub(crate) static ref DIV: Arc<String> = Arc::new("div".into());
    pub(crate) static ref REM: Arc<String> = Arc::new("rem".into());
    pub(crate) static ref POW: Arc<String> = Arc::new("pow".into());
    pub(crate) static ref DOT: Arc<String> = Arc::new("dot".into());
    pub(crate) static ref CROSS: Arc<String> = Arc::new("cross".into());
    pub(crate) static ref NOT: Arc<String> = Arc::new("not".into());
    pub(crate) static ref NEG: Arc<String> = Arc::new("neg".into());
    pub(crate) static ref NORM: Arc<String> = Arc::new("norm".into());
    pub(crate) static ref T: Arc<String> = Arc::new("T".into());
}

/// Type alias for lazy invariants of external functions.
pub type LazyInvariant = &'static [&'static [Lazy]];

/// Lazy invariant to unwrap first argument.
pub static LAZY_UNWRAP_OR: LazyInvariant = &[&[Lazy::UnwrapOk, Lazy::UnwrapSome]];
/// Lazy invariant for `&&`.
pub static LAZY_AND: LazyInvariant = &[&[Lazy::Variable(Variable::Bool(false, None))]];
/// Lazy invariant for `||`.
pub static LAZY_OR: LazyInvariant = &[&[Lazy::Variable(Variable::Bool(true, None))]];
/// Lazy invariant that no arguments have lazy invariants.
pub static LAZY_NO: LazyInvariant = &[];

/// Type alias for Dyon arrays.
pub type Array = Arc<Vec<Variable>>;
/// Type alias for Dyon objects.
pub type Object = Arc<HashMap<Arc<String>, Variable>>;
/// Type alias for Rust objects.
pub type RustObject = Arc<Mutex<dyn Any>>;

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
        let mutex = Arc::try_unwrap(handle).map_err(|_|
            format!("{}\nCan not access Thread because there is \
            more than one reference to it", rt.stack_trace()))?;
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
    Str(Arc<String>),
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
        use Variable::*;

        match *self {
            Str(_) => TEXT_TYPE.clone(),
            F64(_, _) => F64_TYPE.clone(),
            Vec4(_) => VEC4_TYPE.clone(),
            Mat4(_) => MAT4_TYPE.clone(),
            Return => RETURN_TYPE.clone(),
            Bool(_, _) => BOOL_TYPE.clone(),
            Object(_) => OBJECT_TYPE.clone(),
            Array(_) => ARRAY_TYPE.clone(),
            Link(_) => LINK_TYPE.clone(),
            Ref(_) => REF_TYPE.clone(),
            UnsafeRef(_) => UNSAFE_REF_TYPE.clone(),
            RustObject(_) => RUST_OBJECT_TYPE.clone(),
            Option(_) => OPTION_TYPE.clone(),
            Result(_) => RESULT_TYPE.clone(),
            Thread(_) => THREAD_TYPE.clone(),
            Closure(_, _) => CLOSURE_TYPE.clone(),
            In(_) => IN_TYPE.clone(),
        }
    }

    fn deep_clone(&self, stack: &[Variable]) -> Variable {
        use Variable::*;

        match *self {
            F64(_, _) => self.clone(),
            Vec4(_) => self.clone(),
            Mat4(_) => self.clone(),
            Return => self.clone(),
            Bool(_, _) => self.clone(),
            Str(_) => self.clone(),
            Object(ref obj) => {
                let mut res = obj.clone();
                for val in Arc::make_mut(&mut res).values_mut() {
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
            (&Variable::Str(ref a), &Variable::Str(ref b)) => a == b,
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
    /// Relative to function you call from.
    Loaded(isize),
    /// External function with no return value.
    Void(FnVoidRef),
    /// Extern function with return value.
    Return(FnReturnRef),
    /// Extern function with return value and lazy invariant.
    Lazy(FnReturnRef, LazyInvariant),
    /// Extern binary operator.
    BinOp(FnBinOpRef),
    /// Extern unary operator.
    UnOp(FnUnOpRef),
}

/// Refers to an external function.
#[derive(Clone, Copy)]
pub enum FnExt {
    /// External function with no return value.
    Void(fn(&mut Runtime) -> Result<(), String>),
    /// External function with return value.
    Return(fn(&mut Runtime) -> Result<Variable, String>),
    /// External binary operator.
    BinOp(fn(&Variable, &Variable) -> Result<Variable, String>),
    /// External unary operator.
    UnOp(fn(&Variable) -> Result<Variable, String>),
}

impl From<fn(&mut Runtime) -> Result<(), String>> for FnExt {
    fn from(val: fn(&mut Runtime) -> Result<(), String>) -> Self {
        FnExt::Void(val)
    }
}

impl From<fn(&mut Runtime) -> Result<Variable, String>> for FnExt {
    fn from(val: fn(&mut Runtime) -> Result<Variable, String>) -> Self {
        FnExt::Return(val)
    }
}

impl From<fn(&Variable, &Variable) -> Result<Variable, String>> for FnExt {
    fn from(val: fn(&Variable, &Variable) -> Result<Variable, String>) -> Self {
        FnExt::BinOp(val)
    }
}

impl From<fn(&Variable) -> Result<Variable, String>> for FnExt {
    fn from(val: fn(&Variable) -> Result<Variable, String>) -> Self {
        FnExt::UnOp(val)
    }
}

impl fmt::Debug for FnExt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FnExt")
    }
}

/// Used to store direct reference to external function.
#[derive(Copy)]
pub struct FnUnOpRef(pub fn(&Variable) -> Result<Variable, String>);

impl Clone for FnUnOpRef {
    fn clone(&self) -> FnUnOpRef {
        *self
    }
}

impl fmt::Debug for FnUnOpRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FnUnOpRef")
    }
}

/// Used to store direct reference to external function.
#[derive(Copy)]
pub struct FnBinOpRef(pub fn(&Variable, &Variable) -> Result<Variable, String>);

impl Clone for FnBinOpRef {
    fn clone(&self) -> FnBinOpRef {
        *self
    }
}

impl fmt::Debug for FnBinOpRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FnBinOpRef")
    }
}

/// Used to store direct reference to external function.
#[derive(Copy)]
pub struct FnReturnRef(pub fn(&mut Runtime) -> Result<Variable, String>);

impl Clone for FnReturnRef {
    fn clone(&self) -> FnReturnRef {
        *self
    }
}

impl fmt::Debug for FnReturnRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FnExternalRef")
    }
}

/// Used to store direct reference to external function that does not return anything.
#[derive(Copy)]
pub struct FnVoidRef(pub fn(&mut Runtime) -> Result<(), String>);

impl Clone for FnVoidRef {
    fn clone(&self) -> FnVoidRef {
        *self
    }
}

impl fmt::Debug for FnVoidRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FnExternalRef")
    }
}

struct FnExternal {
    namespace: Arc<Vec<Arc<String>>>,
    name: Arc<String>,
    f: FnExt,
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

/// Runs a program using a source file.
pub fn run(source: &str) -> Result<(), String> {
    let mut module = Module::new();
    load(source, &mut module)?;
    let mut runtime = runtime::Runtime::new();
    runtime.run(&Arc::new(module))?;
    Ok(())
}

/// Runs a program from a string.
pub fn run_str(source: &str, d: Arc<String>) -> Result<(), String> {
    let mut module = Module::new();
    load_str(source, d, &mut module)?;
    let mut runtime = runtime::Runtime::new();
    runtime.run(&Arc::new(module))?;
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

    let mut data_file = File::open(source).map_err(|err|
        format!("Could not open `{}`, {}", source, err))?;
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

    let syntax_rules = SYNTAX_RULES.as_ref().map_err(|err| err.clone())?;

    let mut data = vec![];
    parse_errstr(syntax_rules, &d, &mut data).map_err(
        |err| format!("In `{}:`\n{}", source, err)
    )?;

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

    check_ignored_meta_data(conv_res, source, &d, &data, &ignored)
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

    check_ignored_meta_data(conv_res, source, &d, data, &ignored)
}

fn check_ignored_meta_data(
    conv_res: Result<(), ()>,
    source: &str,
    d: &Arc<String>,
    data: &[Range<MetaData>],
    ignored: &[Range],
) -> Result<(), String> {
    use piston_meta::json;

    if !ignored.is_empty() || conv_res.is_err() {
        use std::io::Write;
        use piston_meta::ParseErrorHandler;

        let mut buf: Vec<u8> = vec![];
        if !ignored.is_empty() {
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
        if let Err(()) = conv_res {
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
            println!();
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

    #[test]
    fn expression_size() {
        use std::mem::size_of;
        use super::*;

        assert_eq!(size_of::<ast::Expression>(), 16);
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
