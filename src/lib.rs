#![cfg_attr(test, feature(test))]
extern crate piston_meta;

pub mod ast;
pub mod runtime;
pub mod lifetime;

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

}
