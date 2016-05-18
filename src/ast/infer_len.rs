use std::cell::Cell;
use std::sync::Arc;
use range::Range;

use FnIndex;
use super::{
    Block,
    Call,
    Expression,
    ForN,
    Id,
    Item,
};

pub fn infer(block: &Block, name: &str) -> Option<Expression> {
    let list: Option<(Arc<String>, Range)> = infer_block(block, name);
    let res = list.map(|(list, range)| {
        Expression::Call(Call {
            name: Arc::new("len".into()),
            f_index: Cell::new(FnIndex::None),
            args: vec![
                Expression::Item(Item::from_variable(list, range))
            ],
            source_range: range,
        })
    });
    res
}

fn infer_expr(
    expr: &Expression,
    name: &str
) -> Option<(Arc<String>, Range)> {
    use super::Expression::*;

    match *expr {
        Item(ref item) => {
            if item.ids.len() == 0 { return None; }
            if let Id::Expression(ref expr) = item.ids[0] {
                if let &Expression::Item(ref id) = expr {
                    if &**id.name == name {
                        return Some((
                            item.name.clone(),
                            item.source_range
                        ));
                    }
                }
            }
        }
        BinOp(ref binop_expr) => {
            let left = infer_expr(&binop_expr.left, name);
            if left.is_some() { return left; }
            let right = infer_expr(&binop_expr.right, name);
            if right.is_some() { return right; }
        }
        Assign(ref assign_expr) => {
            let left = infer_expr(&assign_expr.left, name);
            if left.is_some() { return left; }
            let right = infer_expr(&assign_expr.right, name);
            if right.is_some() { return right; }
        }
        Number(_) => {}
        Object(ref obj) => {
            for &(_, ref v) in &obj.key_values {
                let res = infer_expr(v, name);
                if res.is_some() { return res; }
            }
        }
        Array(ref arr) => {
            for expr in &arr.items {
                let res = infer_expr(expr, name);
                if res.is_some() { return res; }
            }
        }
        ArrayFill(ref arr_fill) => {
            let fill = infer_expr(&arr_fill.fill, name);
            if fill.is_some() { return fill; }
            let n = infer_expr(&arr_fill.n, name);
            if n.is_some() { return n; }
        }
        Return(_, ref ret_expr) => {
            let res = infer_expr(ret_expr, name);
            if res.is_some() { return res; }
        }
        ReturnVoid(_) => {}
        Break(_) => {}
        Continue(_) => {}
        Block(ref block) => {
            for expr in &block.expressions {
                let res = infer_expr(expr, name);
                if res.is_some() { return res; }
            }
        }
        Go(ref go) => {
            let res = infer_call(&go.call, name);
            if res.is_some() { return res; }
        }
        Call(ref call) => {
            let res = infer_call(call, name);
            if res.is_some() { return res; }
        }
        Text(_) => {}
        Vec4(_) => {}
        Bool(_) => {}
        For(ref for_expr) => {
            let res = infer_expr(&for_expr.init, name);
            if res.is_some() { return res; }
            let res = infer_expr(&for_expr.cond, name);
            if res.is_some() { return res; }
            let res = infer_expr(&for_expr.step, name);
            if res.is_some() { return res; }
            for expr in &for_expr.block.expressions {
                let res = infer_expr(expr, name);
                if res.is_some() { return res; }
            }
        }
        ForN(ref for_n_expr) => {
            let res = infer_for_n(for_n_expr, name);
            if res.is_some() { return res; }
        }
        Sum(ref for_n_expr) => {
            let res = infer_for_n(for_n_expr, name);
            if res.is_some() { return res; }
        }
        Min(ref for_n_expr) => {
            let res = infer_for_n(for_n_expr, name);
            if res.is_some() { return res; }
        }
        Max(ref for_n_expr) => {
            let res = infer_for_n(for_n_expr, name);
            if res.is_some() { return res; }
        }
        Sift(ref for_n_expr) => {
            let res = infer_for_n(for_n_expr, name);
            if res.is_some() { return res; }
        }
        Any(ref for_n_expr) => {
            let res = infer_for_n(for_n_expr, name);
            if res.is_some() { return res; }
        }
        All(ref for_n_expr) => {
            let res = infer_for_n(for_n_expr, name);
            if res.is_some() { return res; }
        }
        If(ref if_expr) => {
            let res = infer_expr(&if_expr.cond, name);
            if res.is_some() { return res; }
            let res = infer_block(&if_expr.true_block, name);
            if res.is_some() { return res; }
            for (cond, block) in if_expr.else_if_conds.iter()
                .zip(if_expr.else_if_blocks.iter()) {
                let res = infer_expr(cond, name);
                if res.is_some() { return res; }
                let res = infer_block(block, name);
                if res.is_some() { return res; }
            }
            if let Some(ref else_block) = if_expr.else_block {
                let res = infer_block(else_block, name);
                if res.is_some() { return res; }
            }
        }
        Compare(ref cmp_expr) => {
            let left = infer_expr(&cmp_expr.left, name);
            if left.is_some() { return left; }
            let right = infer_expr(&cmp_expr.right, name);
            if right.is_some() { return right; }
        }
        UnOp(ref unop_expr) => {
            let res = infer_expr(&unop_expr.expr, name);
            if res.is_some() { return res; }
        }
        Variable(_, _) => {}
        Try(ref try_expr) => {
            let res = infer_expr(try_expr, name);
            if res.is_some() { return res; }
        }
        // ref x => {panic!("TEST {:?}", x)}
    };
    None
}

fn infer_call(
    call: &Call,
    name: &str
) -> Option<(Arc<String>, Range)> {
    for arg in &call.args {
        let res = infer_expr(arg, name);
        if res.is_some() { return res; }
    }
    None
}

fn infer_for_n(
    for_n_expr: &ForN,
    name: &str
) -> Option<(Arc<String>, Range)> {
    // Check for declaration of same name.
    if &**for_n_expr.name == name {
        return None;
    }
    if let Some(ref start) = for_n_expr.start {
        let res = infer_expr(start, name);
        if res.is_some() { return res; }
    }
    let res = infer_expr(&for_n_expr.end, name);
    if res.is_some() { return res; }
    for expr in &for_n_expr.block.expressions {
        let res = infer_expr(expr, name);
        if res.is_some() { return res; }
    }
    None
}

fn infer_block(
    block: &Block,
    name: &str
) -> Option<(Arc<String>, Range)> {
    for expr in &block.expressions {
        if let &Expression::Assign(ref assign_expr) = expr {
            // Check for declaration of same name.
            if let Expression::Item(ref item) = assign_expr.left {
                if &**item.name == name {
                    return None;
                }
            }
        }
        let res = infer_expr(expr, name);
        if res.is_some() { return res; }
    }
    None
}
