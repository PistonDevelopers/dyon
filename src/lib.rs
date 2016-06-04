#![cfg_attr(test, feature(test))]
extern crate piston_meta;
extern crate rand;
extern crate range;
extern crate read_color;
extern crate hyper;

use std::any::Any;
use std::fmt;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use range::Range;

pub mod ast;
pub mod runtime;
pub mod lifetime;
pub mod intrinsics;
pub mod prelude;
pub mod embed;
pub mod ty;
pub mod link;
pub mod macros;
pub mod vec4;

pub use runtime::Runtime;
pub use prelude::{Lt, Prelude, PreludeFunction};
pub use ty::Type;
pub use link::Link;
pub use vec4::Vec4;

/// A common error message when there is no value on the stack.
pub const TINVOTS: &'static str = "There is no value on the stack";

pub type Array = Arc<Vec<Variable>>;
pub type Object = Arc<HashMap<Arc<String>, Variable>>;
pub type RustObject = Arc<Mutex<Any>>;

#[derive(Debug, Clone)]
pub struct Error {
    pub message: Variable,
    // Extra information to help debug error.
    // Stores error messages for all `?` operators.
    pub trace: Vec<String>,
}

#[derive(Clone)]
pub struct Thread {
    pub handle: Option<Arc<Mutex<JoinHandle<Result<Variable, String>>>>>,
}

impl Thread {
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

#[derive(Debug, Clone)]
pub enum Variable {
    Ref(usize),
    Return,
    Bool(bool),
    F64(f64),
    Vec4([f32; 4]),
    Text(Arc<String>),
    Array(Array),
    Object(Object),
    Link(Link),
    UnsafeRef(UnsafeRef),
    RustObject(RustObject),
    Option(Option<Box<Variable>>),
    Result(Result<Box<Variable>, Box<Error>>),
    Thread(Thread),
}

/// This is requires because `UnsafeRef(*mut Variable)` can not be sent across threads.
/// The lack of `UnsafeRef` variant when sending across threads is guaranteed at language level.
/// The interior of `UnsafeRef` can not be accessed outside this library.
unsafe impl Send for Variable {}

impl Variable {
    fn deep_clone(&self, stack: &Vec<Variable>) -> Variable {
        use Variable::*;

        match *self {
            F64(_) => self.clone(),
            Vec4(_) => self.clone(),
            Return => self.clone(),
            Bool(_) => self.clone(),
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
        }
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Variable) -> bool {
        match (self, other) {
            (&Variable::Return, _) => false,
            (&Variable::Bool(a), &Variable::Bool(b)) => a == b,
            (&Variable::F64(a), &Variable::F64(b)) => a == b,
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

#[derive(Clone, Copy, Debug)]
pub enum FnIndex {
    None,
    /// Relative to function you call from.
    Loaded(isize),
    External(usize),
}

pub struct FnExternal {
    pub name: Arc<String>,
    pub f: fn(&mut Runtime) -> Result<(), String>,
    pub p: PreludeFunction,
}

impl Clone for FnExternal {
    fn clone(&self) -> FnExternal {
        FnExternal {
            name: self.name.clone(),
            f: self.f,
            p: self.p.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Module {
    pub functions: Vec<ast::Function>,
    pub ext_prelude: Vec<FnExternal>,
}

impl Module {
    pub fn new() -> Module {
        Module {
            functions: vec![],
            ext_prelude: vec![],
        }
    }

    pub fn register(&mut self, function: ast::Function) {
        self.functions.push(function);
    }

    /// Find function relative another function index.
    pub fn find_function(&self, name: &Arc<String>, relative: usize) -> FnIndex {
        for (i, f) in self.functions.iter().enumerate().rev() {
            if &f.name == name {
                return FnIndex::Loaded(i as isize - relative as isize);
            }
        }
        for (i, f) in self.ext_prelude.iter().enumerate().rev() {
            if &f.name == name {
                return FnIndex::External(i);
            }
        }
        FnIndex::None
    }

    pub fn error(&self, range: Range, msg: &str, rt: &Runtime) -> String {
        self.error_fnindex(range, msg, rt.call_stack.last().unwrap().index)
    }

    pub fn error_fnindex(&self, range: Range, msg: &str, fnindex: usize) -> String {
        let source = &self.functions[fnindex].source;
        self.error_source(range, msg, source)
    }

    pub fn error_source(&self, range: Range, msg: &str, source: &Arc<String>) -> String {
        use piston_meta::ParseErrorHandler;

        let mut w: Vec<u8> = vec![];
        ParseErrorHandler::new(source)
            .write_msg(&mut w, range, &format!("{}", msg))
            .unwrap();
        String::from_utf8(w).unwrap()
    }

    /// Adds a new extended prelude function.
    pub fn add(
        &mut self,
        name: Arc<String>,
        f: fn(&mut Runtime) -> Result<(), String>,
        prelude_function: PreludeFunction
    ) {
        self.ext_prelude.push(FnExternal {
            name: name.clone(),
            f: f,
            p: prelude_function,
        });
    }
}

/// Runs a program using a syntax file and the source file.
pub fn run(source: &str) -> Result<(), String> {
    let mut module = Module::new();
    try!(load(source, &mut module));
    let mut runtime = runtime::Runtime::new();
    try!(runtime.run(&module));
    Ok(())
}

/// Loads a source from file.
pub fn load(source: &str, module: &mut Module) -> Result<(), String> {
    use std::thread;
    use std::fs::File;
    use std::io::Read;
    use piston_meta::{parse_errstr, syntax_errstr, json};

    let syntax = include_str!("../assets/syntax.txt");
    let syntax_rules = try!(syntax_errstr(syntax));

    let mut data_file = try!(File::open(source).map_err(|err|
        format!("Could not open `{}`, {}", source, err)));
    let mut d = Arc::new(String::new());
    data_file.read_to_string(Arc::make_mut(&mut d)).unwrap();

    let mut data = vec![];
    try!(parse_errstr(&syntax_rules, &d, &mut data).map_err(
        |err| format!("In `{}:`\n{}", source, err)
    ));
    let check_data = data.clone();
    let prelude = Arc::new(Prelude::from_module(module));
    let prelude2 = prelude.clone();

    // Do lifetime checking in parallel directly on meta data.
    let handle = thread::spawn(move || {
        let check_data = check_data;
        lifetime::check(&check_data, &prelude2)
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
    fn bench_push_link_go(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/push_link_go.dyon"));
    }

    #[bench]
    fn bench_push_str(b: &mut Bencher) {
        b.iter(|| run_bench("source/bench/push_str.dyon"));
    }
}
