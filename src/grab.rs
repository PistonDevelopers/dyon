/*
Evaluate grab expressions and return closure where grab expressions
are constants.
*/

use ast;
use runtime::{Flow, Runtime, Side};
use Module;
use Variable;

pub enum Grabbed {
    Variable(Option<Variable>),
    Expression(ast::Expression),
}

pub fn grab_expr(
    rt: &mut Runtime,
    expr: &ast::Expression,
    side: Side,
    module: &Module,
) -> Result<(Grabbed, Flow), String> {
    use ast::Expression as E;

    match expr {
        &E::Grab(ref expr) => {
            // Evaluate the expression and insert it into new AST as constant.
            let v = match try!(rt.expression(expr, side, module)) {
                (Some(x), Flow::Continue) => x,
                (x, Flow::Return) => { return Ok((Grabbed::Variable(x), Flow::Return)); }
                _ => return Err(module.error(expr.source_range(),
                                &format!("{}\nExpected something",
                                    rt.stack_trace()), rt))
            };
            Ok((Grabbed::Expression(E::Variable(expr.source_range(),
                v.deep_clone(&rt.stack))), Flow::Continue))
        }
        &E::Return(ref item_expr, ref expr) => {
            Ok((Grabbed::Expression(E::Return(item_expr.clone(),
                Box::new(match grab_expr(rt, expr, side, module) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                }))),
                Flow::Continue))
        }
        &E::BinOp(ref binop_expr) => {
            Ok((Grabbed::Expression(E::BinOp(Box::new(ast::BinOpExpression {
                op: binop_expr.op.clone(),
                left: match grab_expr(rt, &binop_expr.left, side, module) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                right: match grab_expr(rt, &binop_expr.right, side, module) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                source_range: binop_expr.source_range,
            }))), Flow::Continue))
        }
        &E::Number(_) =>
            Ok((Grabbed::Expression(expr.clone()), Flow::Continue)),
        &E::Item(ref item) => {
            Ok((Grabbed::Expression(E::Item(ast::Item {
                name: item.name.clone(),
                stack_id: item.stack_id.clone(),
                static_stack_id: item.static_stack_id.clone(),
                current: item.current.clone(),
                try: item.try.clone(),
                ids: {
                    let mut new_ids = vec![];
                    for id in &item.ids {
                        new_ids.push(match id {
                            &ast::Id::String(_, _) => id.clone(),
                            &ast::Id::F64(_, _) => id.clone(),
                            &ast::Id::Expression(ref expr) =>
                                match grab_expr(rt, &expr, side, module) {
                                    Ok((Grabbed::Expression(x), Flow::Continue)) =>
                                        ast::Id::Expression(x),
                                    x => return x,
                                },
                        });
                    }
                    new_ids
                },
                try_ids: item.try_ids.clone(),
                source_range: item.source_range.clone(),
            })), Flow::Continue))
        }
        x => panic!("Unimplemented {:#?}", x)
    }
}
