use piston_meta::json;
use std::io;
use ast;
use Runtime;
use Variable;

#[derive(Copy, Clone)]
pub enum EscapeString {
    Json,
    None
}

pub fn write_variable<W>(
    w: &mut W,
    rt: &Runtime,
    v: &Variable,
    escape_string: EscapeString,
    tabs: u32,
) -> Result<(), io::Error>
    where W: io::Write
{
    match *v {
        Variable::Text(ref t) => {
            match escape_string {
                EscapeString::Json => {
                    try!(json::write_string(w, t));
                }
                EscapeString::None => {
                    try!(write!(w, "{}", t))
                }
            }
        }
        Variable::F64(x, _) => {
            try!(write!(w, "{}", x));
        }
        Variable::Vec4(v) => {
            try!(write!(w, "({}, {}", v[0], v[1]));
            if v[2] != 0.0 || v[3] != 0.0 {
                try!(write!(w, ", {}", v[2]));
                if v[3] != 0.0 {
                    try!(write!(w, ", {})", v[3]));
                } else {
                    try!(write!(w, ")"));
                }
            } else {
                try!(write!(w, ")"));
            }
        }
        Variable::Bool(x, _) => {
            try!(write!(w, "{}", x));
        }
        Variable::Ref(ind) => {
            try!(write_variable(w, rt, &rt.stack[ind], escape_string, tabs));
        }
        Variable::Link(ref link) => {
            match escape_string {
                EscapeString::Json => {
                    // Write link items.
                    try!(write!(w, "link {{ "));
                    for slice in &link.slices {
                        for i in slice.start..slice.end {
                            let v = slice.block.var(i);
                            try!(write_variable(w, rt, &v, EscapeString::Json, tabs));
                            try!(write!(w, " "));
                        }
                    }
                    try!(write!(w, "}}"));
                }
                EscapeString::None => {
                    for slice in &link.slices {
                        for i in slice.start..slice.end {
                            let v = slice.block.var(i);
                            try!(write_variable(w, rt, &v, EscapeString::None, tabs));
                        }
                    }
                }
            }
        }
        Variable::Object(ref obj) => {
            try!(write!(w, "{{"));
            let n = obj.len();
            for (i, (k, v)) in obj.iter().enumerate() {
                try!(write!(w, "{}: ", k));
                try!(write_variable(w, rt, v, EscapeString::Json, tabs));
                if i + 1 < n {
                    try!(write!(w, ", "));
                }
            }
            try!(write!(w, "}}"));
        }
        Variable::Array(ref arr) => {
            try!(write!(w, "["));
            let n = arr.len();
            for (i, v) in arr.iter().enumerate() {
                try!(write_variable(w, rt, v, EscapeString::Json, tabs));
                if i + 1 < n {
                    try!(write!(w, ", "));
                }
            }
            try!(write!(w, "]"));
        }
        Variable::Option(ref opt) => {
            match opt {
                &None => {
                    try!(write!(w, "none()"))
                }
                &Some(ref v) => {
                    try!(write!(w, "some("));
                    try!(write_variable(w, rt, v, EscapeString::Json, tabs));
                    try!(write!(w, ")"));
                }
            }
        }
        Variable::Result(ref res) => {
            match res {
                &Err(ref err) => {
                    try!(write!(w, "err("));
                    try!(write_variable(w, rt, &err.message,
                                        EscapeString::Json, tabs));
                    try!(write!(w, ")"));
                }
                &Ok(ref ok) => {
                    try!(write!(w, "ok("));
                    try!(write_variable(w, rt, ok, EscapeString::Json, tabs));
                    try!(write!(w, ")"));
                }
            }
        }
        Variable::Thread(_) => try!(write!(w, "_thread")),
        Variable::Return => try!(write!(w, "_return")),
        Variable::UnsafeRef(_) => try!(write!(w, "_unsafe_ref")),
        Variable::RustObject(_) => try!(write!(w, "_rust_object")),
        Variable::Closure(ref closure, _) => try!(write_closure(w, rt, closure, tabs)),
        // ref x => panic!("Could not print out `{:?}`", x)
    }
    Ok(())
}

pub fn print_variable(rt: &Runtime, v: &Variable, escape_string: EscapeString) {
    write_variable(&mut io::stdout(), rt, v, escape_string, 0).unwrap();
}

fn write_tabs<W: io::Write>(w: &mut W, tabs: u32) -> Result<(), io::Error> {
    for _ in 0..tabs {
        try!(write!(w, "    "));
    }
    Ok(())
}

pub fn write_closure<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    closure: &ast::Closure,
    tabs: u32
) -> Result<(), io::Error> {
    try!(write!(w, "\\("));
    for (i, arg) in closure.args.iter().enumerate() {
        try!(write_arg(w, arg));
        if i + 1 < closure.args.len() {
            try!(write!(w, ", "));
        }
    }
    try!(write!(w, ") = "));
    try!(write_expr(w, rt, &closure.expr, tabs));
    Ok(())
}

pub fn write_arg<W: io::Write>(
    w: &mut W,
    arg: &ast::Arg
) -> Result<(), io::Error> {
    write!(w, "{}: {}", arg.name, arg.ty.description())
}

pub fn write_expr<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    expr: &ast::Expression,
    tabs: u32,
) -> Result<(), io::Error> {
    use ast::Expression as E;

    match expr {
        &E::BinOp(ref binop) => try!(write_binop(w, rt, binop, tabs)),
        &E::Item(ref item) => try!(write_item(w, rt, item, tabs)),
        &E::Number(ref number) => try!(write!(w, "{}", number.num)),
        &E::Text(ref text) => try!(json::write_string(w, &text.text)),
        &E::Bool(ref b) => try!(write!(w, "{}", b.val)),
        &E::Variable(_, ref var) => try!(write_variable(w, rt, var, EscapeString::Json, tabs)),
        &E::Link(ref link) => try!(write_link(w, rt, link, tabs)),
        &E::Object(ref obj) => try!(write_obj(w, rt, obj, tabs)),
        &E::Array(ref arr) => try!(write_arr(w, rt, arr, tabs)),
        &E::ArrayFill(ref arr_fill) => try!(write_arr_fill(w, rt, arr_fill, tabs)),
        &E::Call(ref call) => try!(write_call(w, rt, call, tabs)),
        &E::Return(ref ret, ref expr) => {
            try!(write_expr(w, rt, ret, tabs));
            try!(write!(w, " "));
            try!(write_expr(w, rt, expr, tabs));
        }
        &E::ReturnVoid(_) => try!(write!(w, "return")),
        &E::Break(ref br) => {
            if let Some(ref label) = br.label {
                try!(write!(w, "break '{}", label));
            } else {
                try!(write!(w, "break"));
            }
        }
        &E::Continue(ref c) => {
            if let Some(ref label) = c.label {
                try!(write!(w, "continue '{}", label));
            } else {
                try!(write!(w, "continue"));
            }
        }
        &E::Block(ref b) => try!(write_block(w, rt, b, tabs)),
        &E::Go(ref go) => {
            try!(write!(w, "go "));
            try!(write_call(w, rt, &go.call, tabs));
        }
        &E::Assign(ref assign) => try!(write_assign(w, rt, assign, tabs)),
        &E::Vec4(ref vec4) => try!(write_vec4(w, rt, vec4, tabs)),
        &E::For(ref f) => try!(write_for(w, rt, f, tabs)),
        &E::Compare(ref comp) => try!(write_compare(w, rt, comp, tabs)),
        &E::ForN(ref for_n) => {
            try!(write!(w, "for "));
            try!(write_for_n(w, rt, for_n, tabs));
        }
        &E::Sum(ref for_n) => {
            try!(write!(w, "sum "));
            try!(write_for_n(w, rt, for_n, tabs));
        }
        &E::SumVec4(ref for_n) => {
            try!(write!(w, "sum_vec4 "));
            try!(write_for_n(w, rt, for_n, tabs));
        }
        &E::Prod(ref for_n) => {
            try!(write!(w, "prod "));
            try!(write_for_n(w, rt, for_n, tabs));
        }
        &E::Min(ref for_n) => {
            try!(write!(w, "min "));
            try!(write_for_n(w, rt, for_n, tabs));
        }
        &E::Max(ref for_n) => {
            try!(write!(w, "max "));
            try!(write_for_n(w, rt, for_n, tabs));
        }
        &E::Sift(ref for_n) => {
            try!(write!(w, "sift "));
            try!(write_for_n(w, rt, for_n, tabs));
        }
        &E::Any(ref for_n) => {
            try!(write!(w, "any "));
            try!(write_for_n(w, rt, for_n, tabs));
        }
        &E::All(ref for_n) => {
            try!(write!(w, "all "));
            try!(write_for_n(w, rt, for_n, tabs));
        }
        &E::If(ref if_expr) => try!(write_if(w, rt, if_expr, tabs)),
        &E::UnOp(ref unop) => try!(write_unop(w, rt, unop, tabs)),
        &E::Try(ref expr) => {
            try!(write_expr(w, rt, expr, tabs));
            try!(write!(w, "?"));
        }
        &E::Swizzle(ref swizzle) => try!(write_swizzle(w, rt, swizzle, tabs)),
        &E::Closure(ref closure) => try!(write_closure(w, rt, closure, tabs)),
        &E::Grab(ref grab) => try!(write_grab(w, rt, grab, tabs)),
        &E::CallClosure(ref call) => try!(write_call_closure(w, rt, call, tabs)),
        // x => panic!("Unimplemented `{:#?}`", x),
    }
    Ok(())
}

pub fn write_block<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    block: &ast::Block,
    tabs: u32,
) -> Result<(), io::Error> {
    match block.expressions.len() {
        0 => {
            try!(write!(w, "{{}}"));
        }
        1 => {
            try!(write!(w, "{{ "));
            try!(write_expr(w, rt, &block.expressions[0], tabs + 1));
            try!(write!(w, " }}"));
        }
        _ => {
            try!(writeln!(w, "{{"));
            for expr in &block.expressions {
                try!(write_tabs(w, tabs + 1));
                try!(write_expr(w, rt, expr, tabs + 1));
                try!(writeln!(w, ""));
            }
            try!(write_tabs(w, tabs));
            try!(write!(w, "}}"));
        }
    }
    Ok(())
}

pub fn write_binop<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    binop: &ast::BinOpExpression,
    tabs: u32,
) -> Result<(), io::Error> {
    try!(write_expr(w, rt, &binop.left, tabs));
    try!(write!(w, " {} ", binop.op.symbol()));
    try!(write_expr(w, rt, &binop.right, tabs));
    Ok(())
}

pub fn write_unop<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    unop: &ast::UnOpExpression,
    tabs: u32,
) -> Result<(), io::Error> {
    use ast::UnOp::*;

    match unop.op {
        Norm => {
            try!(write!(w, "|"));
            try!(write_expr(w, rt, &unop.expr, tabs));
            try!(write!(w, "|"));
        }
        Not => {
            try!(write!(w, "!"));
            try!(write_expr(w, rt, &unop.expr, tabs));
        }
        Neg => {
            try!(write!(w, "-"));
            try!(write_expr(w, rt, &unop.expr, tabs));
        }
    }
    Ok(())
}

pub fn write_item<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    item: &ast::Item,
    tabs: u32,
) -> Result<(), io::Error> {
    use ast::Id;

    if item.current {
        try!(write!(w, "~ "));
    }
    try!(write!(w, "{}", item.name));
    for (i, id) in item.ids.iter().enumerate() {
        match id {
            &Id::String(_, ref prop) => try!(write!(w, ".{}", prop)),
            &Id::F64(_, ind) => try!(write!(w, "[{}]", ind)),
            &Id::Expression(ref expr) => try!(write_expr(w, rt, expr, tabs)),
        }
        if item.try_ids.iter().any(|&tr| tr == i) {
            try!(write!(w, "?"));
        }
    }
    Ok(())
}

pub fn write_link<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    link: &ast::Link,
    tabs: u32,
) -> Result<(), io::Error> {
    try!(write!(w, "link {{ "));
    for item in &link.items {
        try!(write_expr(w, rt, item, tabs));
        try!(write!(w, " "));
    }
    try!(write!(w, "}}"));
    Ok(())
}

pub fn write_obj<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    obj: &ast::Object,
    tabs: u32,
) -> Result<(), io::Error> {
    try!(write!(w, "{{"));
    for (i, key_value) in obj.key_values.iter().enumerate() {
        try!(write!(w, "{}: ", key_value.0));
        try!(write_expr(w, rt, &key_value.1, tabs + 1));
        if i + 1 < obj.key_values.len() {
            try!(write!(w, ", "));
        }
    }
    try!(write!(w, "}}"));
    Ok(())
}

pub fn write_call<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    call: &ast::Call,
    tabs: u32,
) -> Result<(), io::Error> {
    try!(write!(w, "{}(", call.name));
    for (i, arg) in call.args.iter().enumerate() {
        try!(write_expr(w, rt, arg, tabs));
        if i + 1 < call.args.len() {
            try!(write!(w, ", "));
        }
    }
    try!(write!(w, ")"));
    Ok(())
}

pub fn write_call_closure<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    call: &ast::CallClosure,
    tabs: u32,
) -> Result<(), io::Error> {
    try!(write!(w, "\\"));
    try!(write_item(w, rt, &call.item, tabs));
    try!(write!(w, "("));
    for (i, arg) in call.args.iter().enumerate() {
        try!(write_expr(w, rt, arg, tabs + 1));
        if i + 1 < call.args.len() {
            try!(write!(w, ", "));
        }
    }
    try!(write!(w, ")"));
    Ok(())
}

pub fn write_arr<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    arr: &ast::Array,
    tabs: u32,
) -> Result<(), io::Error> {
    try!(write!(w, "["));
    for (i, item) in arr.items.iter().enumerate() {
        try!(write_expr(w, rt, item, tabs + 1));
        if i + 1 < arr.items.len() {
            try!(write!(w, ", "));
        }
    }
    try!(write!(w, "]"));
    Ok(())
}

pub fn write_arr_fill<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    arr_fill: &ast::ArrayFill,
    tabs: u32,
) -> Result<(), io::Error> {
    try!(write!(w, "["));
    try!(write_expr(w, rt, &arr_fill.fill, tabs + 1));
    try!(write!(w, ", "));
    try!(write_expr(w, rt, &arr_fill.n, tabs + 1));
    try!(write!(w, "]"));
    Ok(())
}

pub fn write_assign<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    assign: &ast::Assign,
    tabs: u32,
) -> Result<(), io::Error> {
    try!(write_expr(w, rt, &assign.left, tabs));
    try!(write!(w, " {} ", assign.op.symbol()));
    try!(write_expr(w, rt, &assign.right, tabs));
    Ok(())
}

pub fn write_vec4<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    vec4: &ast::Vec4,
    tabs: u32,
) -> Result<(), io::Error> {
    let mut n = vec4.args.len();
    for expr in vec4.args.iter().rev() {
        if let &ast::Expression::Number(ref num) = expr {
            if num.num == 0.0 {
                n -= 1;
                continue;
            }
        }
        break;
    }
    try!(write!(w, "("));
    for (i, expr) in vec4.args[0..n].iter().enumerate() {
        try!(write_expr(w, rt, expr, tabs));
        if i + 1 < n {
            try!(write!(w, ", "));
        }
        if i + 1 == n && i == 0 {
            try!(write!(w, ","));
        }
    }
    try!(write!(w, ")"));
    Ok(())
}

pub fn write_swizzle<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    swizzle: &ast::Swizzle,
    tabs: u32,
) -> Result<(), io::Error> {
    let comp = |ind: usize| {
        match ind {
            0 => "x",
            1 => "y",
            2 => "z",
            3 => "w",
            _ => panic!("Wrong swizzle component"),
        }
    };
    try!(write!(w, "{}", comp(swizzle.sw0)));
    try!(write!(w, "{}", comp(swizzle.sw1)));
    if let Some(sw2) = swizzle.sw2 {
        try!(write!(w, "{}", comp(sw2)));
    }
    if let Some(sw3) = swizzle.sw3 {
        try!(write!(w, "{}", comp(sw3)));
    }
    try!(write!(w, " "));
    try!(write_expr(w, rt, &swizzle.expr, tabs));
    Ok(())
}

pub fn write_for<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    f: &ast::For,
    tabs: u32,
) -> Result<(), io::Error> {
    if let ast::Expression::Block(ref b) = f.init {
        if b.expressions.len() == 0 {
            if let ast::Expression::Bool(ref b) = f.cond {
                if b.val {
                    if let ast::Expression::Block(ref b) = f.step {
                        if b.expressions.len() == 0 {
                            try!(write!(w, "loop "));
                            try!(write_block(w, rt, &f.block, tabs + 1));
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    try!(write!(w, "for "));
    try!(write_expr(w, rt, &f.init, tabs));
    try!(write!(w, "; "));
    try!(write_expr(w, rt, &f.cond, tabs));
    try!(write!(w, "; "));
    try!(write_expr(w, rt, &f.step, tabs));
    try!(write!(w, " "));
    try!(write_block(w, rt, &f.block, tabs + 1));
    Ok(())
}

pub fn write_compare<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    comp: &ast::Compare,
    tabs: u32,
) -> Result<(), io::Error> {
    try!(write_expr(w, rt, &comp.left, tabs));
    try!(write!(w, " {} ", comp.op.symbol()));
    try!(write_expr(w, rt, &comp.right, tabs));
    Ok(())
}

pub fn write_for_n<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    for_n: &ast::ForN,
    tabs: u32
) -> Result<(), io::Error> {
    try!(write!(w, "{} ", for_n.name));
    if let Some(ref start) = for_n.start {
        try!(write!(w, "["));
        try!(write_expr(w, rt, start, tabs));
        try!(write!(w, ", "));
        try!(write_expr(w, rt, &for_n.end, tabs));
        try!(write!(w, ") "));
    } else {
        try!(write_expr(w, rt, &for_n.end, tabs));
        try!(write!(w, " "));
    }
    try!(write_block(w, rt, &for_n.block, tabs + 1));
    Ok(())
}

pub fn write_if<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    if_expr: &ast::If,
    tabs: u32,
) -> Result<(), io::Error> {
    try!(write!(w, "if "));
    try!(write_expr(w, rt, &if_expr.cond, tabs));
    try!(write!(w, " "));
    try!(write_block(w, rt, &if_expr.true_block, tabs + 1));
    for (else_if_cond, else_if_block) in if_expr.else_if_conds.iter()
        .zip(if_expr.else_if_blocks.iter()) {
        try!(write!(w, " else if "));
        try!(write_expr(w, rt, else_if_cond, tabs));
        try!(write!(w, " "));
        try!(write_block(w, rt, else_if_block, tabs + 1));
    }
    if let Some(ref else_block) = if_expr.else_block {
        try!(write!(w, " else "));
        try!(write_block(w, rt, else_block, tabs + 1));
    }
    Ok(())
}

pub fn write_grab<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    grab: &ast::Grab,
    tabs: u32,
) -> Result<(), io::Error> {
    if grab.level != 1 {
        try!(write!(w, "grab '{} ", grab.level));
    } else {
        try!(write!(w, "grab "));
    }
    try!(write_expr(w, rt, &grab.expr, tabs));
    Ok(())
}
