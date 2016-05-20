use std::sync::Arc;

use super::{
    Array,
    ArrayFill,
    Assign,
    Block,
    BinOpExpression,
    Call,
    Compare,
    Expression,
    For,
    ForN,
    Go,
    Id,
    If,
    Item,
    Object,
    Number,
    UnOpExpression,
    Vec4,
};

/// Replaces an item with a number.
/// Returns `(true, new_expression)` if item found declared with same name.
/// Returns `(false, cloned_expression)` if there was no item with same name.
/// The flag is used to just clone the rest of expressions in a block.
pub fn number(expr: &Expression, name: &Arc<String>, val: f64) -> Expression {
    use super::Expression as E;

    match *expr {
        E::Number(_) => expr.clone(),
        E::BinOp(ref bin_op_expr) => {
            E::BinOp(Box::new(BinOpExpression {
                op: bin_op_expr.op,
                left: number(&bin_op_expr.left, name, val),
                right: number(&bin_op_expr.right, name, val),
                source_range: bin_op_expr.source_range,
            }))
        }
        E::Item(ref item) => {
            if &item.name == name {
                E::Number(Number {
                    num: val,
                    source_range: item.source_range,
                })
            } else {
                let mut new_ids: Vec<Id> = vec![];
                for id in &item.ids {
                    if let &Id::Expression(ref expr) = id {
                        new_ids.push(Id::Expression(number(expr, name, val)));
                    } else {
                        new_ids.push(id.clone());
                    }
                }
                E::Item(Item {
                    name: item.name.clone(),
                    stack_id: item.stack_id.clone(),
                    static_stack_id: item.static_stack_id.clone(),
                    try: item.try.clone(),
                    ids: new_ids,
                    try_ids: item.try_ids.clone(),
                    source_range: item.source_range,
                })
            }
        }
        E::Block(ref block) => {
            E::Block(number_block(block, name, val))
        }
        E::Assign(ref assign_expr) => {
            E::Assign(Box::new(Assign {
                op: assign_expr.op.clone(),
                left: number(&assign_expr.left, name, val),
                right: number(&assign_expr.right, name, val),
                source_range: assign_expr.source_range,
            }))
        }
        E::Object(ref obj_expr) => {
            let mut new_key_values: Vec<(Arc<String>, Expression)> = vec![];
            for key_value in &obj_expr.key_values {
                new_key_values.push((key_value.0.clone(),
                    number(&key_value.1, name, val)));
            }
            E::Object(Box::new(Object {
                key_values: new_key_values,
                source_range: obj_expr.source_range,
            }))
        }
        E::Call(ref call_expr) => {
            E::Call(number_call(call_expr, name, val))
        }
        E::Array(ref array_expr) => {
            let mut new_items: Vec<Expression> = vec![];
            for item in &array_expr.items {
                new_items.push(number(item, name, val));
            }
            E::Array(Box::new(Array {
                items: new_items,
                source_range: array_expr.source_range,
            }))
        }
        E::ArrayFill(ref array_fill_expr) => {
            E::ArrayFill(Box::new(ArrayFill {
                fill: number(&array_fill_expr.fill, name, val),
                n: number(&array_fill_expr.n, name, val),
                source_range: array_fill_expr.source_range,
            }))
        }
        E::Return(ref ret, ref ret_expr) => {
            E::Return(ret.clone(), Box::new(number(ret_expr, name, val)))
        }
        E::ReturnVoid(_) => expr.clone(),
        E::Break(_) => expr.clone(),
        E::Continue(_) => expr.clone(),
        E::Go(ref go) => {
            E::Go(Box::new(Go {
                call: number_call(&go.call, name, val),
                source_range: go.source_range,
            }))
        }
        E::Text(_) => expr.clone(),
        E::Vec4(ref vec4_expr) => {
            let mut new_args: Vec<Expression> = vec![];
            for arg in &vec4_expr.args {
                new_args.push(number(arg, name, val));
            }
            E::Vec4(Vec4 {
                args: new_args,
                source_range: vec4_expr.source_range,
            })
        }
        E::Bool(_) => expr.clone(),
        E::For(ref for_expr) => {
            let mut init: Option<Expression> = None;
            if let Expression::Assign(ref assign_expr) = for_expr.init {
                // Check for declaration of same name.
                if let Expression::Item(ref item) = assign_expr.left {
                    if &item.name == name {
                        init = Some(Expression::Assign(Box::new(Assign {
                            op: assign_expr.op.clone(),
                            left: assign_expr.left.clone(),
                            right: number(&assign_expr.right, name, val),
                            source_range: assign_expr.source_range,
                        })));
                    }
                }
            }
            if let Some(init) = init {
                E::For(Box::new(For {
                    label: for_expr.label.clone(),
                    init: init,
                    cond: for_expr.cond.clone(),
                    step: for_expr.step.clone(),
                    block: for_expr.block.clone(),
                    source_range: for_expr.source_range,
                }))
            } else {
                E::For(Box::new(For {
                    label: for_expr.label.clone(),
                    init: number(&for_expr.init, name, val),
                    cond: number(&for_expr.cond, name, val),
                    step: number(&for_expr.step, name, val),
                    block: number_block(&for_expr.block, name, val),
                    source_range: for_expr.source_range,
                }))
            }
        }
        E::ForN(ref for_n_expr) => {
            E::ForN(Box::new(number_for_n(for_n_expr, name, val)))
        }
        E::Sum(ref for_n_expr) => {
            E::ForN(Box::new(number_for_n(for_n_expr, name, val)))
        }
        E::SumVec4(ref for_n_expr) => {
            E::ForN(Box::new(number_for_n(for_n_expr, name, val)))
        }
        E::Min(ref for_n_expr) => {
            E::ForN(Box::new(number_for_n(for_n_expr, name, val)))
        }
        E::Max(ref for_n_expr) => {
            E::ForN(Box::new(number_for_n(for_n_expr, name, val)))
        }
        E::Sift(ref for_n_expr) => {
            E::ForN(Box::new(number_for_n(for_n_expr, name, val)))
        }
        E::Any(ref for_n_expr) => {
            E::ForN(Box::new(number_for_n(for_n_expr, name, val)))
        }
        E::All(ref for_n_expr) => {
            E::ForN(Box::new(number_for_n(for_n_expr, name, val)))
        }
        E::If(ref if_expr) => {
            let mut new_else_if_conds: Vec<Expression> = vec![];
            for else_if_cond in &if_expr.else_if_conds {
                new_else_if_conds.push(number(else_if_cond, name, val));
            }
            let mut new_else_if_blocks: Vec<Block> = vec![];
            for else_if_block in &if_expr.else_if_blocks {
                new_else_if_blocks.push(number_block(else_if_block, name, val));
            }
            E::If(Box::new(If {
                cond: number(&if_expr.cond, name, val),
                true_block: number_block(&if_expr.true_block, name, val),
                else_if_conds: new_else_if_conds,
                else_if_blocks: new_else_if_blocks,
                else_block: if_expr.else_block.as_ref()
                    .map(|else_block| number_block(else_block, name, val)),
                source_range: if_expr.source_range,
            }))
        }
        E::Compare(ref cmp_expr) => {
            E::Compare(Box::new(Compare {
                op: cmp_expr.op.clone(),
                left: number(&cmp_expr.left, name, val),
                right: number(&cmp_expr.right, name, val),
                source_range: cmp_expr.source_range,
            }))
        }
        E::UnOp(ref unop_expr) => {
            E::UnOp(Box::new(UnOpExpression {
                op: unop_expr.op.clone(),
                expr: number(&unop_expr.expr, name, val),
                source_range: unop_expr.source_range,
            }))
        }
        E::Variable(_, _) => expr.clone(),
        E::Try(ref expr) => E::Try(Box::new(number(expr, name, val))),
    }
}

fn number_call(call_expr: &Call, name: &Arc<String>, val: f64) -> Call {
    let mut new_args: Vec<Expression> = vec![];
    for arg in &call_expr.args {
        new_args.push(number(arg, name, val));
    }
    Call {
        name: call_expr.name.clone(),
        args: new_args,
        f_index: call_expr.f_index.clone(),
        source_range: call_expr.source_range,
    }
}

fn number_block(block: &Block, name: &Arc<String>, val: f64) -> Block {
    let mut new_expressions: Vec<Expression> = vec![];
    let mut just_clone = false;
    for expr in &block.expressions {
        if just_clone {
            new_expressions.push(expr.clone());
        } else {
            if let &Expression::Assign(ref assign_expr) = expr {
                // Check for declaration of same name.
                if let Expression::Item(ref item) = assign_expr.left {
                    if &item.name == name {
                        new_expressions.push(Expression::Assign(Box::new(Assign {
                            op: assign_expr.op.clone(),
                            left: assign_expr.left.clone(),
                            right: number(&assign_expr.right, name, val),
                            source_range: assign_expr.source_range,
                        })));
                        just_clone = true;
                        continue;
                    }
                }
            }
            new_expressions.push(number(expr, name, val));
        }
    }
    Block {
        expressions: new_expressions,
        source_range: block.source_range,
    }
}

fn number_for_n(for_n_expr: &ForN, name: &Arc<String>, val: f64) -> ForN {
    if &for_n_expr.name == name {
        for_n_expr.clone()
    } else {
        ForN {
            label: for_n_expr.label.clone(),
            name: for_n_expr.name.clone(),
            start: for_n_expr.start.as_ref()
                .map(|start| number(start, name, val)),
            end: number(&for_n_expr.end, name, val),
            block: number_block(&for_n_expr.block, name, val),
            source_range: for_n_expr.source_range,
        }
    }
}
