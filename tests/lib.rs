extern crate piston_meta;
extern crate dyon;

use dyon::*;

pub fn test_src(source: &str) {
    let mut module = Module::new();
    load(source, &mut module).unwrap_or_else(|err| {
        panic!("In `{}`:\n{}", source, err);
    });
}

pub fn test_fail_src(source: &str) {
    let mut module = Module::new();
    match load(source, &mut module) {
        Ok(_) => panic!("`{}` should fail", source),
        Err(err) => {
            if err.starts_with(&format!("Could not open `{}`", source)) {
                panic!("{}", err)
            }
        }
    };
}

pub fn debug_src(source: &str) {
    let mut module = Module::new();
    load(source, &mut module).unwrap_or_else(|err| {
        panic!("In `{}`:\n{}", source, err);
    });
    panic!("{:?}", module.functions);
}

#[test]
fn test_syntax() {
    test_src("source/syntax/main.dyon");
    test_src("source/syntax/args.dyon");
    test_src("source/syntax/id.dyon");
    test_src("source/syntax/call.dyon");
    test_src("source/syntax/array.dyon");
    test_src("source/syntax/prop.dyon");
    test_src("source/syntax/for.dyon");
    test_src("source/syntax/compare.dyon");
    test_src("source/syntax/add.dyon");
    test_src("source/syntax/mul.dyon");
    test_src("source/syntax/pow.dyon");
    test_src("source/syntax/add_mul.dyon");
    test_src("source/syntax/mul_add.dyon");
    test_src("source/syntax/pos_len.dyon");
    test_src("source/syntax/if.dyon");
    test_src("source/syntax/else_if.dyon");
    test_src("source/syntax/assign_if.dyon");
    test_src("source/syntax/new_pos.dyon");
    test_src("source/syntax/lifetime.dyon");
    test_fail_src("source/syntax/lifetime_2.dyon");
    test_fail_src("source/syntax/lifetime_3.dyon");
    test_fail_src("source/syntax/lifetime_4.dyon");
    test_fail_src("source/syntax/lifetime_5.dyon");
    test_src("source/syntax/lifetime_6.dyon");
    test_src("source/syntax/lifetime_7.dyon");
    test_src("source/syntax/lifetime_8.dyon");
    test_fail_src("source/syntax/lifetime_9.dyon");
    test_src("source/syntax/insert.dyon");
    test_src("source/syntax/named_call.dyon");
    test_src("source/syntax/max_min.dyon");
    test_src("source/syntax/return_void.dyon");
    test_src("source/syntax/typeof.dyon");
    test_src("source/syntax/load_module.dyon");
    test_src("source/syntax/println_colon.dyon");
    test_src("source/syntax/neg.dyon");
    test_src("source/syntax/some.dyon");
    test_src("source/syntax/pop.dyon");
    test_src("source/syntax/accessor.dyon");
    test_src("source/syntax/sum.dyon");
    test_src("source/syntax/min_max.dyon");
    test_src("source/syntax/vec4.dyon");
    test_src("source/syntax/color.dyon");
    test_src("source/syntax/parens.dyon");
}

#[test]
fn test_typechk() {
    test_fail_src("source/typechk/opt.dyon");
    test_fail_src("source/typechk/return.dyon");
    test_fail_src("source/typechk/return_2.dyon");
    test_fail_src("source/typechk/return_3.dyon");
    test_fail_src("source/typechk/return_4.dyon");
    test_fail_src("source/typechk/return_5.dyon");
    test_fail_src("source/typechk/return_6.dyon");
    test_fail_src("source/typechk/return_7.dyon");
    test_fail_src("source/typechk/return_8.dyon");
    test_src("source/typechk/add.dyon");
    test_src("source/typechk/mat_expr.dyon");
    test_src("source/typechk/or.dyon");
    test_fail_src("source/typechk/or_2.dyon");
    test_fail_src("source/typechk/mul.dyon");
    test_fail_src("source/typechk/pow.dyon");
    test_src("source/typechk/pow_2.dyon");
    test_fail_src("source/typechk/pow_3.dyon");
    test_fail_src("source/typechk/call.dyon");
    test_src("source/typechk/obj.dyon");
    test_fail_src("source/typechk/go.dyon");
    test_fail_src("source/typechk/threads.dyon");
    test_fail_src("source/typechk/unused_result.dyon");
    test_fail_src("source/typechk/unused_result_2.dyon");
    test_src("source/typechk/res.dyon");
}

#[test]
fn test_functions() {
    test_src("source/functions/functions.dyon");
}

#[test]
fn test_error() {
    test_src("source/error/propagate.dyon");
    test_src("source/error/call.dyon");
    test_src("source/error/named_call.dyon");
    test_src("source/error/if.dyon");
    test_src("source/error/trace.dyon");
    test_src("source/error/unwrap_err.dyon");
    test_src("source/error/option.dyon");
}
