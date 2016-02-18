#![cfg_attr(test, feature(test))]
extern crate piston_meta;
extern crate rand;
extern crate range;

use std::any::Any;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use range::Range;

use lifetime::Prelude;

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

#[derive(Debug)]
pub struct Module {
    pub source: Option<String>,
    pub functions: HashMap<Arc<String>, Arc<ast::Function>>,
}

impl Module {
    pub fn new() -> Module {
        Module {
            source: None,
            functions: HashMap::new(),
        }
    }

    pub fn register(&mut self, function: Arc<ast::Function>) {
        self.functions.insert(function.name.clone(), function);
    }

    pub fn error(&self, range: Range, msg: &str) -> String {
        use piston_meta::ParseErrorHandler;

        let mut w: Vec<u8> = vec![];
        ParseErrorHandler::new(&self.source.as_ref().unwrap())
            .write_msg(&mut w, range, &format!("{}", msg))
            .unwrap();
        String::from_utf8(w).unwrap()
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
    let mut d = String::new();
    data_file.read_to_string(&mut d).unwrap();
    module.source = Some(d.clone());

    let mut data = vec![];
    try!(parse_errstr(&syntax_rules, &d, &mut data));
    let check_data = data.clone();
    let prelude = Prelude::from_module(module);

    // Do lifetime checking in parallel directly on meta data.
    let handle = thread::spawn(move || {
        let check_data = check_data;
        let prelude = prelude;
        lifetime::check(&check_data, &prelude)
    });

    // Convert to AST.
    let mut ignored = vec![];
    ast::convert(&data, &mut ignored, module).unwrap();

    // Check that lifetime checking succeeded.
    match handle.join().unwrap() {
        Ok(()) => {}
        Err(err_msg) => {
            let (range, msg) = err_msg.decouple();
            return Err(format!("In `{}`:\n{}", source, module.error(range, &msg)))
        }
    }

    if ignored.len() > 0 {
        use std::io::Write;

        let mut buf: Vec<u8> = vec![];
        writeln!(&mut buf, "Some meta data was ignored in the syntax").unwrap();
        writeln!(&mut buf, "START IGNORED").unwrap();
        for r in &ignored {
            json::write(&mut buf, &data[r.iter()]).unwrap();
        }
        writeln!(&mut buf, "END IGNORED").unwrap();
        return Err(String::from_utf8(buf).unwrap());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::run;
    use self::test::Bencher;

    #[bench]
    fn bench_add_two(b: &mut Bencher) {
        b.iter(||
            run("source/bench/add.rs")
        );
    }

    #[bench]
    fn bench_main(b: &mut Bencher) {
        b.iter(||
            run("source/bench/main.rs")
        );
    }

    #[bench]
    fn bench_array(b: &mut Bencher) {
        b.iter(||
            run("source/bench/array.rs")
        );
    }

    #[bench]
    fn bench_object(b: &mut Bencher) {
        b.iter(||
            run("source/bench/object.rs")
        );
    }

    #[bench]
    fn bench_call(b: &mut Bencher) {
        b.iter(||
            run("source/bench/call.rs")
        );
    }

    #[bench]
    fn bench_n_body(b: &mut Bencher) {
        b.iter(||
            run("source/bench/n_body.rs")
        );
    }
}
