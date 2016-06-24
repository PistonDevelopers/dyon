/*
Evaluate grab expressions and return closure where grab expressions
are constants.
*/

use ast;
use runtime::{Flow, Runtime, Side};
use Module;
use Variable;

#[derive(Debug)]
pub enum Grabbed {
    Variable(Option<Variable>),
    Expression(ast::Expression),
    Block(ast::Block),
    Item(ast::Item),
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
        &E::Number(_) |
        &E::Closure(_) =>
            Ok((Grabbed::Expression(expr.clone()), Flow::Continue)),
        &E::Item(ref item) => match grab_item(rt, item, side, module) {
            Ok((Grabbed::Item(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::Item(x)), Flow::Continue))
            }
            x => return x,
        },
        &E::Block(ref block) => match grab_block(rt, block, side, module) {
            Ok((Grabbed::Block(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::Block(x)), Flow::Continue))
            }
            x => return x,
        },
        &E::Assign(ref assign) => {
            Ok((Grabbed::Expression(E::Assign(Box::new(ast::Assign {
                op: assign.op.clone(),
                left: match grab_expr(rt, &assign.left, side, module) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                right: match grab_expr(rt, &assign.right, side, module) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                source_range: assign.source_range.clone(),
            }))), Flow::Continue))
        },
        &E::Compare(ref compare) => {
            Ok((Grabbed::Expression(E::Compare(Box::new(ast::Compare {
                op: compare.op.clone(),
                left: match grab_expr(rt, &compare.left, side, module) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                right: match grab_expr(rt, &compare.right, side, module) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                source_range: compare.source_range.clone(),
            }))), Flow::Continue))
        }
        &E::If(ref if_expr) => {
            Ok((Grabbed::Expression(E::If(Box::new(ast::If {
                cond: match grab_expr(rt, &if_expr.cond, side, module) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                true_block: match grab_block(rt, &if_expr.true_block, side, module) {
                    Ok((Grabbed::Block(x), Flow::Continue)) => x,
                    x => return x,
                },
                else_if_conds: {
                    let mut new_else_if_conds = vec![];
                    for else_if_cond in &if_expr.else_if_conds {
                        new_else_if_conds.push(match grab_expr(rt, else_if_cond, side, module) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_else_if_conds
                },
                else_if_blocks: {
                    let mut new_else_if_blocks = vec![];
                    for else_if_block in &if_expr.else_if_blocks {
                        new_else_if_blocks.push(match grab_block(rt, else_if_block, side, module) {
                            Ok((Grabbed::Block(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_else_if_blocks
                },
                else_block: match if_expr.else_block {
                    None => None,
                    Some(ref else_block) => {
                        match grab_block(rt, &else_block, side, module) {
                            Ok((Grabbed::Block(x), Flow::Continue)) => Some(x),
                            x => return x,
                        }
                    }
                },
                source_range: if_expr.source_range.clone(),
            }))), Flow::Continue))
        },
        &E::CallClosure(ref call_closure) => {
            Ok((Grabbed::Expression(E::CallClosure(Box::new(ast::CallClosure {
                item: match grab_item(rt, &call_closure.item, side, module) {
                    Ok((Grabbed::Item(x), Flow::Continue)) => x,
                    x => return x,
                },
                args: {
                    let mut new_args = vec![];
                    for arg in &call_closure.args {
                        new_args.push(match grab_expr(rt, arg, side, module) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_args
                },
                source_range: call_closure.source_range.clone(),
            }))), Flow::Continue))
        }
        x => panic!("Unimplemented {:#?}", x)
    }
}

fn grab_block(
    rt: &mut Runtime,
    block: &ast::Block,
    side: Side,
    module: &Module,
) -> Result<(Grabbed, Flow), String> {
    Ok((Grabbed::Block(ast::Block {
        expressions: {
            let mut new_expressions = vec![];
            for expr in &block.expressions {
                new_expressions.push(match grab_expr(rt, expr, side, module) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                });
            }
            new_expressions
        },
        source_range: block.source_range.clone()
    }), Flow::Continue))
}

fn grab_item(
    rt: &mut Runtime,
    item: &ast::Item,
    side: Side,
    module: &Module,
) -> Result<(Grabbed, Flow), String> {
    Ok((Grabbed::Item(ast::Item {
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
    }), Flow::Continue))
}
