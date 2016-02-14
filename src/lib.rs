#![cfg_attr(test, feature(test))]
extern crate piston_meta;

use std::any::Any;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub mod ast;
pub mod runtime;
pub mod lifetime;
pub mod intrinsics;

pub type Object = HashMap<Arc<String>, Variable>;
pub type Array = Vec<Variable>;

#[derive(Debug, Clone)]
pub enum Variable {
    Return,
    Bool(bool),
    F64(f64),
    Text(Arc<String>),
    Object(Object),
    Array(Vec<Variable>),
    Ref(usize),
    UnsafeRef(*mut Variable),
    RustObject(Arc<Mutex<Any>>),
}

pub struct Module {
    pub functions: HashMap<Arc<String>, Arc<ast::Function>>,
}

impl Module {
    pub fn new() -> Module {
        Module {
            functions: HashMap::new(),
        }
    }

    pub fn register(&mut self, function: Arc<ast::Function>) {
        self.functions.insert(function.name.clone(), function);
    }
}

/// Runs a program using a syntax file and the source file.
pub fn run(syntax: &str, source: &str) {
    use std::thread;
    use piston_meta::{json, load_syntax_data};

    let data = load_syntax_data(syntax, source);
    let check_data = data.clone();

    // Do lifetime checking in parallel directly on meta data.
    let handle = thread::spawn(move || {
        let check_data = check_data;
        lifetime::check(&check_data)
    });

    // Convert to AST.
    let mut ignored = vec![];
    let ast = ast::convert(&data, &mut ignored).unwrap();

    // Check that lifetime checking succeeded.
    match handle.join().unwrap() {
        Ok(()) => {}
        Err(msg) => panic!(msg)
    }

    let mut runtime = runtime::Runtime::new();
    if ignored.len() > 0 {
        println!("START IGNORED");
        if ignored.len() > 0 {
            for r in &ignored {
                json::print(&data[r.iter()]);
            }
        }
        println!("END IGNORED");
        panic!("Some meta data was ignored in the syntax");
    }
    runtime.run(&ast);
}

/// Loads a source from file.
pub fn load(source: &str, prelude: &[Arc<ast::Function>])
-> Result<Module, String> {
    use std::thread;
    use std::fs::File;
    use std::io::Read;
    use piston_meta::{parse_errstr, syntax_errstr};

    let syntax = include_str!("../assets/syntax.txt");
    let syntax_rules = try!(syntax_errstr(syntax));

    let mut data_file = File::open(source).unwrap();
    let mut d = String::new();
    data_file.read_to_string(&mut d).unwrap();

    let mut data = vec![];
    try!(parse_errstr(&syntax_rules, &d, &mut data));
    let check_data = data.clone();

    // Do lifetime checking in parallel directly on meta data.
    let handle = thread::spawn(move || {
        let check_data = check_data;
        lifetime::check(&check_data)
    });

    // Convert to AST.
    let mut ignored = vec![];
    let mut ast = ast::convert(&data, &mut ignored).unwrap();

    // Check that lifetime checking succeeded.
    match handle.join().unwrap() {
        Ok(()) => {}
        Err(msg) => return Err(msg)
    }

    if ignored.len() > 0 {
        unimplemented!();
    }

    for f in prelude {
        ast.register(f.clone());
    }

    Ok(ast)
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::run;
    use self::test::Bencher;

    #[bench]
    fn bench_add_two(b: &mut Bencher) {
        b.iter(||
            run("assets/syntax.txt", "source/bench/add.rs")
        );
    }

    #[bench]
    fn bench_main(b: &mut Bencher) {
        b.iter(||
            run("assets/syntax.txt", "source/bench/main.rs")
        );
    }

    #[bench]
    fn bench_array(b: &mut Bencher) {
        b.iter(||
            run("assets/syntax.txt", "source/bench/array.rs")
        );
    }

    #[bench]
    fn bench_object(b: &mut Bencher) {
        b.iter(||
            run("assets/syntax.txt", "source/bench/object.rs")
        );
    }

    #[bench]
    fn bench_call(b: &mut Bencher) {
        b.iter(||
            run("assets/syntax.txt", "source/bench/call.rs")
        );
    }
}
