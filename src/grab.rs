/*
Evaluate grab expressions and return closure where grab expressions
are constants.
*/

use ast;
use runtime::{Flow, Runtime, Side};
use std::sync::Arc;
use Variable;

#[derive(Debug)]
pub enum Grabbed {
    Variable(Option<Variable>),
    Expression(ast::Expression),
    Block(ast::Block),
    Item(ast::Item),
    ForN(ast::ForN),
}

pub fn grab_expr(
    level: u16,
    rt: &mut Runtime,
    expr: &ast::Expression,
    side: Side,
) -> Result<(Grabbed, Flow), String> {
    use ast::Expression as E;

    match *expr {
        E::Grab(ref grab) => {
            if grab.level == level {
                // Evaluate the expression and insert it into new AST as constant.
                let v = match rt.expression(&grab.expr, side)? {
                    (Some(x), Flow::Continue) => x,
                    (x, Flow::Return) => {
                        return Ok((Grabbed::Variable(x), Flow::Return));
                    }
                    _ => {
                        return Err(rt.module.error(
                            expr.source_range(),
                            &format!("{}\nExpected something", rt.stack_trace()),
                            rt,
                        ))
                    }
                };
                Ok((
                    Grabbed::Expression(E::Variable(Box::new((
                        expr.source_range(),
                        v.deep_clone(&rt.stack),
                    )))),
                    Flow::Continue,
                ))
            } else {
                Ok((Grabbed::Expression(expr.clone()), Flow::Continue))
            }
        }
        E::Return(ref expr) => Ok((
            Grabbed::Expression(E::Return(Box::new(
                match grab_expr(level, rt, expr, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
            ))),
            Flow::Continue,
        )),
        E::Try(ref expr) => Ok((
            Grabbed::Expression(E::Try(Box::new(match grab_expr(level, rt, expr, side) {
                Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                x => return x,
            }))),
            Flow::Continue,
        )),
        E::ReturnVoid(_) | E::Break(_) | E::Continue(_) | E::Variable(_) => {
            Ok((Grabbed::Expression(expr.clone()), Flow::Continue))
        }
        E::Closure(ref closure) => Ok((
            Grabbed::Expression(E::Closure(Arc::new(ast::Closure {
                file: closure.file.clone(),
                source: closure.source.clone(),
                args: closure.args.clone(),
                currents: closure.currents.clone(),
                expr: match grab_expr(level + 1, rt, &closure.expr, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                ret: closure.ret.clone(),
                source_range: closure.source_range,
            }))),
            Flow::Continue,
        )),
        E::Item(ref item) => match grab_item(level, rt, item, side) {
            Ok((Grabbed::Item(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::Item(Box::new(x))), Flow::Continue))
            }
            x => x,
        },
        E::Block(ref block) => match grab_block(level, rt, block, side) {
            Ok((Grabbed::Block(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::Block(Box::new(x))), Flow::Continue))
            }
            x => x,
        },
        E::Assign(ref assign) => Ok((
            Grabbed::Expression(E::Assign(Box::new(ast::Assign {
                op: assign.op,
                left: match grab_expr(level, rt, &assign.left, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                right: match grab_expr(level, rt, &assign.right, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                source_range: assign.source_range,
            }))),
            Flow::Continue,
        )),
        E::If(ref if_expr) => Ok((
            Grabbed::Expression(E::If(Box::new(ast::If {
                cond: match grab_expr(level, rt, &if_expr.cond, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                true_block: match grab_block(level, rt, &if_expr.true_block, side) {
                    Ok((Grabbed::Block(x), Flow::Continue)) => x,
                    x => return x,
                },
                else_if_conds: {
                    let mut new_else_if_conds = vec![];
                    for else_if_cond in &if_expr.else_if_conds {
                        new_else_if_conds.push(match grab_expr(level, rt, else_if_cond, side) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_else_if_conds
                },
                else_if_blocks: {
                    let mut new_else_if_blocks = vec![];
                    for else_if_block in &if_expr.else_if_blocks {
                        new_else_if_blocks.push(match grab_block(level, rt, else_if_block, side) {
                            Ok((Grabbed::Block(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_else_if_blocks
                },
                else_block: match if_expr.else_block {
                    None => None,
                    Some(ref else_block) => match grab_block(level, rt, else_block, side) {
                        Ok((Grabbed::Block(x), Flow::Continue)) => Some(x),
                        x => return x,
                    },
                },
                source_range: if_expr.source_range,
            }))),
            Flow::Continue,
        )),
        E::Go(ref go) => {
            let call = &go.call;
            Ok((
                Grabbed::Expression(E::Go(Box::new(ast::Go {
                    call: ast::Call {
                        args: {
                            let mut new_args = vec![];
                            for arg in &call.args {
                                new_args.push(match grab_expr(level, rt, arg, side) {
                                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                                    x => return x,
                                });
                            }
                            new_args
                        },
                        info: call.info.clone(),
                        f_index: call.f_index,
                        custom_source: call.custom_source.clone(),
                    },
                    source_range: go.source_range,
                }))),
                Flow::Continue,
            ))
        }
        E::Call(ref call) => Ok((
            Grabbed::Expression(E::Call(Box::new(ast::Call {
                args: {
                    let mut new_args = vec![];
                    for arg in &call.args {
                        new_args.push(match grab_expr(level, rt, arg, side) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_args
                },
                info: call.info.clone(),
                f_index: call.f_index,
                custom_source: call.custom_source.clone(),
            }))),
            Flow::Continue,
        )),
        E::CallVoid(ref call) => Ok((
            Grabbed::Expression(E::CallVoid(Box::new(ast::CallVoid {
                args: {
                    let mut new_args = vec![];
                    for arg in &call.args {
                        new_args.push(match grab_expr(level, rt, arg, side) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_args
                },
                info: call.info.clone(),
                fun: call.fun,
            }))),
            Flow::Continue,
        )),
        E::CallReturn(ref call) => Ok((
            Grabbed::Expression(E::CallReturn(Box::new(ast::CallReturn {
                args: {
                    let mut new_args = vec![];
                    for arg in &call.args {
                        new_args.push(match grab_expr(level, rt, arg, side) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_args
                },
                info: call.info.clone(),
                fun: call.fun,
            }))),
            Flow::Continue,
        )),
        E::CallBinOp(ref call) => Ok((
            Grabbed::Expression(E::CallBinOp(Box::new(ast::CallBinOp {
                left: match grab_expr(level, rt, &call.left, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                right: match grab_expr(level, rt, &call.right, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                info: call.info.clone(),
                fun: call.fun,
            }))),
            Flow::Continue,
        )),
        E::CallUnOp(ref call) => Ok((
            Grabbed::Expression(E::CallUnOp(Box::new(ast::CallUnOp {
                arg: match grab_expr(level, rt, &call.arg, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                info: call.info.clone(),
                fun: call.fun,
            }))),
            Flow::Continue,
        )),
        E::CallLazy(ref call) => Ok((
            Grabbed::Expression(E::CallLazy(Box::new(ast::CallLazy {
                args: {
                    let mut new_args = vec![];
                    for arg in &call.args {
                        new_args.push(match grab_expr(level, rt, arg, side) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_args
                },
                lazy_inv: call.lazy_inv,
                info: call.info.clone(),
                fun: call.fun,
            }))),
            Flow::Continue,
        )),
        E::CallLoaded(ref call) => Ok((
            Grabbed::Expression(E::CallLoaded(Box::new(ast::CallLoaded {
                args: {
                    let mut new_args = vec![];
                    for arg in &call.args {
                        new_args.push(match grab_expr(level, rt, arg, side) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_args
                },
                info: call.info.clone(),
                custom_source: call.custom_source.clone(),
                fun: call.fun,
            }))),
            Flow::Continue,
        )),
        E::CallClosure(ref call_closure) => Ok((
            Grabbed::Expression(E::CallClosure(Box::new(ast::CallClosure {
                item: match grab_item(level, rt, &call_closure.item, side) {
                    Ok((Grabbed::Item(x), Flow::Continue)) => x,
                    x => return x,
                },
                args: {
                    let mut new_args = vec![];
                    for arg in &call_closure.args {
                        new_args.push(match grab_expr(level, rt, arg, side) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_args
                },
                source_range: call_closure.source_range,
            }))),
            Flow::Continue,
        )),
        E::ForN(ref for_n) => match grab_for_n(level, rt, for_n, side) {
            Ok((Grabbed::ForN(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::ForN(Box::new(x))), Flow::Continue))
            }
            x => x,
        },
        E::Sum(ref for_n) => match grab_for_n(level, rt, for_n, side) {
            Ok((Grabbed::ForN(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::Sum(Box::new(x))), Flow::Continue))
            }
            x => x,
        },
        E::Prod(ref for_n) => match grab_for_n(level, rt, for_n, side) {
            Ok((Grabbed::ForN(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::Prod(Box::new(x))), Flow::Continue))
            }
            x => x,
        },
        E::Min(ref for_n) => match grab_for_n(level, rt, for_n, side) {
            Ok((Grabbed::ForN(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::Min(Box::new(x))), Flow::Continue))
            }
            x => x,
        },
        E::Max(ref for_n) => match grab_for_n(level, rt, for_n, side) {
            Ok((Grabbed::ForN(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::Max(Box::new(x))), Flow::Continue))
            }
            x => x,
        },
        E::Any(ref for_n) => match grab_for_n(level, rt, for_n, side) {
            Ok((Grabbed::ForN(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::Any(Box::new(x))), Flow::Continue))
            }
            x => x,
        },
        E::All(ref for_n) => match grab_for_n(level, rt, for_n, side) {
            Ok((Grabbed::ForN(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::All(Box::new(x))), Flow::Continue))
            }
            x => x,
        },
        E::LinkFor(ref for_n) => match grab_for_n(level, rt, for_n, side) {
            Ok((Grabbed::ForN(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::LinkFor(Box::new(x))), Flow::Continue))
            }
            x => x,
        },
        E::SumVec4(ref for_n) => match grab_for_n(level, rt, for_n, side) {
            Ok((Grabbed::ForN(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::SumVec4(Box::new(x))), Flow::Continue))
            }
            x => x,
        },
        E::ProdVec4(ref for_n) => match grab_for_n(level, rt, for_n, side) {
            Ok((Grabbed::ForN(x), Flow::Continue)) => Ok((
                Grabbed::Expression(E::ProdVec4(Box::new(x))),
                Flow::Continue,
            )),
            x => x,
        },
        E::Sift(ref for_n) => match grab_for_n(level, rt, for_n, side) {
            Ok((Grabbed::ForN(x), Flow::Continue)) => {
                Ok((Grabbed::Expression(E::Sift(Box::new(x))), Flow::Continue))
            }
            x => x,
        },
        E::Vec4(ref vec4) => Ok((
            Grabbed::Expression(E::Vec4(Box::new(ast::Vec4 {
                args: {
                    let mut new_args = vec![];
                    for arg in &vec4.args {
                        new_args.push(match grab_expr(level, rt, arg, side) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_args
                },
                source_range: vec4.source_range,
            }))),
            Flow::Continue,
        )),
        E::Mat4(ref mat4) => Ok((
            Grabbed::Expression(E::Mat4(Box::new(ast::Mat4 {
                args: {
                    let mut new_args = vec![];
                    for arg in &mat4.args {
                        new_args.push(match grab_expr(level, rt, arg, side) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_args
                },
                source_range: mat4.source_range,
            }))),
            Flow::Continue,
        )),
        E::Link(ref link) => Ok((
            Grabbed::Expression(E::Link(Box::new(ast::Link {
                items: {
                    let mut new_items = vec![];
                    for item in &link.items {
                        new_items.push(match grab_expr(level, rt, item, side) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_items
                },
                source_range: link.source_range,
            }))),
            Flow::Continue,
        )),
        E::Object(ref obj) => Ok((
            Grabbed::Expression(E::Object(Box::new(ast::Object {
                key_values: {
                    let mut new_key_values = vec![];
                    for key_value in &obj.key_values {
                        new_key_values.push((
                            key_value.0.clone(),
                            match grab_expr(level, rt, &key_value.1, side) {
                                Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                                x => return x,
                            },
                        ));
                    }
                    new_key_values
                },
                source_range: obj.source_range,
            }))),
            Flow::Continue,
        )),
        E::Array(ref arr) => Ok((
            Grabbed::Expression(E::Array(Box::new(ast::Array {
                items: {
                    let mut new_items = vec![];
                    for item in &arr.items {
                        new_items.push(match grab_expr(level, rt, item, side) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                            x => return x,
                        });
                    }
                    new_items
                },
                source_range: arr.source_range,
            }))),
            Flow::Continue,
        )),
        E::ArrayFill(ref arr_fill) => Ok((
            Grabbed::Expression(E::ArrayFill(Box::new(ast::ArrayFill {
                fill: match grab_expr(level, rt, &arr_fill.fill, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                n: match grab_expr(level, rt, &arr_fill.n, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                source_range: arr_fill.source_range,
            }))),
            Flow::Continue,
        )),
        E::For(ref for_expr) => Ok((
            Grabbed::Expression(E::For(Box::new(ast::For {
                init: match grab_expr(level, rt, &for_expr.init, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                cond: match grab_expr(level, rt, &for_expr.cond, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                step: match grab_expr(level, rt, &for_expr.step, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                block: match grab_block(level, rt, &for_expr.block, side) {
                    Ok((Grabbed::Block(x), Flow::Continue)) => x,
                    x => return x,
                },
                label: for_expr.label.clone(),
                source_range: for_expr.source_range,
            }))),
            Flow::Continue,
        )),
        E::ForIn(ref for_in_expr) => Ok((
            Grabbed::Expression(E::ForIn(Box::new(ast::ForIn {
                name: for_in_expr.name.clone(),
                iter: match grab_expr(level, rt, &for_in_expr.iter, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                block: match grab_block(level, rt, &for_in_expr.block, side) {
                    Ok((Grabbed::Block(x), Flow::Continue)) => x,
                    x => return x,
                },
                label: for_in_expr.label.clone(),
                source_range: for_in_expr.source_range,
            }))),
            Flow::Continue,
        )),
        E::SumIn(ref for_in_expr) => Ok((
            Grabbed::Expression(E::SumIn(Box::new(ast::ForIn {
                name: for_in_expr.name.clone(),
                iter: match grab_expr(level, rt, &for_in_expr.iter, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                block: match grab_block(level, rt, &for_in_expr.block, side) {
                    Ok((Grabbed::Block(x), Flow::Continue)) => x,
                    x => return x,
                },
                label: for_in_expr.label.clone(),
                source_range: for_in_expr.source_range,
            }))),
            Flow::Continue,
        )),
        E::ProdIn(ref for_in_expr) => Ok((
            Grabbed::Expression(E::ProdIn(Box::new(ast::ForIn {
                name: for_in_expr.name.clone(),
                iter: match grab_expr(level, rt, &for_in_expr.iter, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                block: match grab_block(level, rt, &for_in_expr.block, side) {
                    Ok((Grabbed::Block(x), Flow::Continue)) => x,
                    x => return x,
                },
                label: for_in_expr.label.clone(),
                source_range: for_in_expr.source_range,
            }))),
            Flow::Continue,
        )),
        E::MinIn(ref for_in_expr) => Ok((
            Grabbed::Expression(E::MinIn(Box::new(ast::ForIn {
                name: for_in_expr.name.clone(),
                iter: match grab_expr(level, rt, &for_in_expr.iter, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                block: match grab_block(level, rt, &for_in_expr.block, side) {
                    Ok((Grabbed::Block(x), Flow::Continue)) => x,
                    x => return x,
                },
                label: for_in_expr.label.clone(),
                source_range: for_in_expr.source_range,
            }))),
            Flow::Continue,
        )),
        E::MaxIn(ref for_in_expr) => Ok((
            Grabbed::Expression(E::MaxIn(Box::new(ast::ForIn {
                name: for_in_expr.name.clone(),
                iter: match grab_expr(level, rt, &for_in_expr.iter, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                block: match grab_block(level, rt, &for_in_expr.block, side) {
                    Ok((Grabbed::Block(x), Flow::Continue)) => x,
                    x => return x,
                },
                label: for_in_expr.label.clone(),
                source_range: for_in_expr.source_range,
            }))),
            Flow::Continue,
        )),
        E::AnyIn(ref for_in_expr) => Ok((
            Grabbed::Expression(E::AnyIn(Box::new(ast::ForIn {
                name: for_in_expr.name.clone(),
                iter: match grab_expr(level, rt, &for_in_expr.iter, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                block: match grab_block(level, rt, &for_in_expr.block, side) {
                    Ok((Grabbed::Block(x), Flow::Continue)) => x,
                    x => return x,
                },
                label: for_in_expr.label.clone(),
                source_range: for_in_expr.source_range,
            }))),
            Flow::Continue,
        )),
        E::AllIn(ref for_in_expr) => Ok((
            Grabbed::Expression(E::AllIn(Box::new(ast::ForIn {
                name: for_in_expr.name.clone(),
                iter: match grab_expr(level, rt, &for_in_expr.iter, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                block: match grab_block(level, rt, &for_in_expr.block, side) {
                    Ok((Grabbed::Block(x), Flow::Continue)) => x,
                    x => return x,
                },
                label: for_in_expr.label.clone(),
                source_range: for_in_expr.source_range,
            }))),
            Flow::Continue,
        )),
        E::SiftIn(ref for_in_expr) => Ok((
            Grabbed::Expression(E::SiftIn(Box::new(ast::ForIn {
                name: for_in_expr.name.clone(),
                iter: match grab_expr(level, rt, &for_in_expr.iter, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                block: match grab_block(level, rt, &for_in_expr.block, side) {
                    Ok((Grabbed::Block(x), Flow::Continue)) => x,
                    x => return x,
                },
                label: for_in_expr.label.clone(),
                source_range: for_in_expr.source_range,
            }))),
            Flow::Continue,
        )),
        E::LinkIn(ref for_in_expr) => Ok((
            Grabbed::Expression(E::LinkIn(Box::new(ast::ForIn {
                name: for_in_expr.name.clone(),
                iter: match grab_expr(level, rt, &for_in_expr.iter, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                block: match grab_block(level, rt, &for_in_expr.block, side) {
                    Ok((Grabbed::Block(x), Flow::Continue)) => x,
                    x => return x,
                },
                label: for_in_expr.label.clone(),
                source_range: for_in_expr.source_range,
            }))),
            Flow::Continue,
        )),
        E::Swizzle(ref swizzle) => Ok((
            Grabbed::Expression(E::Swizzle(Box::new(ast::Swizzle {
                sw0: swizzle.sw0,
                sw1: swizzle.sw1,
                sw2: swizzle.sw2,
                sw3: swizzle.sw3,
                expr: match grab_expr(level, rt, &swizzle.expr, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                source_range: swizzle.source_range,
            }))),
            Flow::Continue,
        )),
        E::TryExpr(ref try_expr) => Ok((
            Grabbed::Expression(E::TryExpr(Box::new(ast::TryExpr {
                expr: match grab_expr(level, rt, &try_expr.expr, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                    x => return x,
                },
                source_range: try_expr.source_range,
            }))),
            Flow::Continue,
        )),
        E::In(_) => Ok((Grabbed::Expression(expr.clone()), Flow::Continue)),
    }
}

fn grab_block(
    level: u16,
    rt: &mut Runtime,
    block: &ast::Block,
    side: Side,
) -> Result<(Grabbed, Flow), String> {
    Ok((
        Grabbed::Block(ast::Block {
            expressions: {
                let mut new_expressions = vec![];
                for expr in &block.expressions {
                    new_expressions.push(match grab_expr(level, rt, expr, side) {
                        Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                        x => return x,
                    });
                }
                new_expressions
            },
            source_range: block.source_range,
        }),
        Flow::Continue,
    ))
}

fn grab_item(
    level: u16,
    rt: &mut Runtime,
    item: &ast::Item,
    side: Side,
) -> Result<(Grabbed, Flow), String> {
    Ok((
        Grabbed::Item(ast::Item {
            name: item.name.clone(),
            stack_id: item.stack_id.clone(),
            static_stack_id: item.static_stack_id.clone(),
            current: item.current,
            try: item.try,
            ids: {
                let mut new_ids = vec![];
                for id in &item.ids {
                    new_ids.push(match *id {
                        ast::Id::String(_, _) => id.clone(),
                        ast::Id::F64(_, _) => id.clone(),
                        ast::Id::Expression(ref expr) => match grab_expr(level, rt, expr, side) {
                            Ok((Grabbed::Expression(x), Flow::Continue)) => ast::Id::Expression(x),
                            x => return x,
                        },
                    });
                }
                new_ids
            },
            try_ids: item.try_ids.clone(),
            source_range: item.source_range,
        }),
        Flow::Continue,
    ))
}

fn grab_for_n(
    level: u16,
    rt: &mut Runtime,
    for_n: &ast::ForN,
    side: Side,
) -> Result<(Grabbed, Flow), String> {
    Ok((
        Grabbed::ForN(ast::ForN {
            name: for_n.name.clone(),
            start: match for_n.start {
                None => None,
                Some(ref start) => match grab_expr(level, rt, start, side) {
                    Ok((Grabbed::Expression(x), Flow::Continue)) => Some(x),
                    x => return x,
                },
            },
            end: match grab_expr(level, rt, &for_n.end, side) {
                Ok((Grabbed::Expression(x), Flow::Continue)) => x,
                x => return x,
            },
            block: match grab_block(level, rt, &for_n.block, side) {
                Ok((Grabbed::Block(x), Flow::Continue)) => x,
                x => return x,
            },
            label: for_n.label.clone(),
            source_range: for_n.source_range,
        }),
        Flow::Continue,
    ))
}
