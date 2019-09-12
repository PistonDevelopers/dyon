use std::sync::Arc;

use runtime::{Flow, Runtime, Side};
use ast;
use prelude::{Lt, Prelude, Dfn};

use Variable;
use Type;
use dyon_std::*;

const NEXT: usize = 0;

const TABLE: &[(usize, fn(
        &mut Runtime,
        &ast::Call,
    ) -> Result<Option<Variable>, String>)]
= &[
    (NEXT, next),
];

pub(crate) fn standard(f: &mut Prelude) {
    f.intrinsic(Arc::new("next".into()), NEXT, Dfn {
        lts: vec![Lt::Default],
        tys: vec![Type::in_ty()],
        ret: Type::Any
    });
}

pub(crate) fn call_standard(
    rt: &mut Runtime,
    index: usize,
    call: &ast::Call
) -> Result<(Option<Variable>, Flow), String> {
    for arg in &call.args {
        match rt.expression(arg, Side::Right)? {
            (x, Flow::Return) => { return Ok((x, Flow::Return)); }
            (Some(v), Flow::Continue) => rt.stack.push(v),
            _ => return Err(rt.module.error(arg.source_range(),
                    &format!("{}\nExpected something. \
                    Expression did not return a value.",
                    rt.stack_trace()), rt))
        };
    }
    let (ind, f) = TABLE[index];
    debug_assert!(ind == index);
    let expect = (f)(rt, call)?;
    Ok((expect, Flow::Continue))
}
