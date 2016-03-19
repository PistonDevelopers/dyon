extern crate piston_meta;
extern crate dyon;

use dyon::*;

pub fn test_src(source: &str) {
    let mut module = Module::new();
    load(source, &mut module).unwrap_or_else(|err| {
        panic!("{}", err);
    });
}

pub fn debug_src(source: &str) {
    let mut module = Module::new();
    load(source, &mut module).unwrap_or_else(|err| {
        panic!("{}", err);
    });
    panic!("{:?}", module.functions);
}

#[test]
fn test_main() {
    test_src("source/main.rs");
}

#[test]
fn test_args() {
    test_src("source/args.rs");
}

#[test]
fn test_id() {
    test_src("source/id.rs");
}

#[test]
fn test_call() {
    test_src("source/call.rs");
}

#[test]
fn test_prop() {
    test_src("source/prop.rs");
}

#[test]
fn test_for() {
    test_src("source/for.rs");
}

#[test]
fn test_compare() {
    test_src("source/compare.rs");
}

#[test]
fn test_add() {
    test_src("source/add.rs");
}

#[test]
fn test_mul() {
    test_src("source/mul.rs");
}

#[test]
fn test_pow() {
    test_src("source/pow.rs");
}

#[test]
fn test_add_mul() {
    test_src("source/add_mul.rs");
}

#[test]
fn test_mul_add() {
    test_src("source/mul_add.rs");
}

#[test]
fn test_pos_len() {
    test_src("source/pos_len.rs");
}

#[test]
fn test_if() {
    test_src("source/if.rs");
}

#[test]
fn test_else_if() {
    test_src("source/else_if.rs");
}

#[test]
fn test_assign_if() {
    test_src("source/assign_if.rs");
}

#[test]
fn test_new_pos() {
    test_src("source/new_pos.rs");
}

#[test]
fn test_lifetime() {
    test_src("source/lifetime.rs");
}

#[test]
fn test_lifetime_6() {
    test_src("source/lifetime_6.rs");
}

#[test]
fn test_insert() {
    test_src("source/insert.rs");
}

#[test]
fn test_named_call() {
    test_src("source/named_call.rs");
}

#[test]
fn test_max_min() {
    test_src("source/max_min.rs");
}

#[test]
fn test_return_void() {
    test_src("source/return_void.rs");
}

#[test]
fn test_typeof() {
    test_src("source/typeof.rs");
}

#[test]
fn test_load_module() {
    test_src("source/load_module.rs");
}

#[test]
fn test_println_colon() {
    test_src("source/println_colon.rs");
}

#[test]
fn test_print_functions() {
    test_src("source/functions/print_functions.rs");
}

#[test]
fn test_some() {
    test_src("source/some.rs");
}

#[test]
fn test_error_propagate() {
    test_src("source/error/propagate.rs");
}

#[test]
fn test_error_call() {
    test_src("source/error/call.rs");
}

#[test]
fn test_error_named_call() {
    test_src("source/error/named_call.rs");
}

#[test]
fn test_error_if() {
    test_src("source/error/if.rs");
}

#[test]
fn test_error_trace() {
    test_src("source/error/trace.rs");
}

#[test]
fn test_error_unwrap_err() {
    test_src("source/error/unwrap_err.rs");
}
