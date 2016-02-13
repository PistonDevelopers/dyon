extern crate piston_meta;
extern crate dynamo;

use piston_meta::*;
use dynamo::*;

pub fn test_src(source: &str) {
    let data = load_syntax_data("assets/syntax.txt", source);
    let mut ignored = vec![];
    let _ = ast::convert(&data, &mut ignored).unwrap();
}

pub fn debug_src(source: &str) {
    let data = load_syntax_data("assets/syntax.txt", source);
    json::print(&data);
    let mut ignored = vec![];
    let functions = ast::convert(&data, &mut ignored).unwrap();
    panic!("{:?}", functions);
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
