use piston_meta::json;
use std::io;
use ast;
use Runtime;
use Variable;

#[derive(Copy, Clone)]
pub(crate) enum EscapeString {
    Json,
    None
}

pub(crate) fn write_variable<W>(
    w: &mut W,
    rt: &Runtime,
    v: &Variable,
    escape_string: EscapeString,
    tabs: u32,
) -> Result<(), io::Error>
    where W: io::Write
{
    match *v {
        Variable::Str(ref t) => {
            match escape_string {
                EscapeString::Json => {
                    json::write_string(w, t)?;
                }
                EscapeString::None => {
                    write!(w, "{}", t)?
                }
            }
        }
        Variable::F64(x, _) => {
            write!(w, "{}", x)?;
        }
        Variable::Vec4(v) => {
            write!(w, "({}, {}", v[0], v[1])?;
            if v[2] != 0.0 || v[3] != 0.0 {
                write!(w, ", {}", v[2])?;
                if v[3] != 0.0 {
                    write!(w, ", {})", v[3])?;
                } else {
                    write!(w, ")")?;
                }
            } else {
                write!(w, ")")?;
            }
        }
        Variable::Mat4(ref m) => {
            write!(w, "mat4 {{{},{},{},{}; {},{},{},{}; {},{},{},{}; {},{},{},{}}}",
                m[0][0], m[1][0], m[2][0], m[3][0],
                m[0][1], m[1][1], m[2][1], m[3][1],
                m[0][2], m[1][2], m[2][2], m[3][2],
                m[0][3], m[1][3], m[2][3], m[3][3]
            )?;
        }
        Variable::Bool(x, _) => {
            write!(w, "{}", x)?;
        }
        Variable::Ref(ind) => {
            write_variable(w, rt, &rt.stack[ind], escape_string, tabs)?;
        }
        Variable::Link(ref link) => {
            match escape_string {
                EscapeString::Json => {
                    // Write link items.
                    write!(w, "link {{ ")?;
                    for slice in &link.slices {
                        for i in slice.start..slice.end {
                            let v = slice.block.var(i);
                            write_variable(w, rt, &v, EscapeString::Json, tabs)?;
                            write!(w, " ")?;
                        }
                    }
                    write!(w, "}}")?;
                }
                EscapeString::None => {
                    for slice in &link.slices {
                        for i in slice.start..slice.end {
                            let v = slice.block.var(i);
                            write_variable(w, rt, &v, EscapeString::None, tabs)?;
                        }
                    }
                }
            }
        }
        Variable::Object(ref obj) => {
            write!(w, "{{")?;
            let n = obj.len();
            for (i, (k, v)) in obj.iter().enumerate() {
                if k.chars().all(|c| c.is_alphanumeric()) {
                    write!(w, "{}: ", k)?;
                } else {
                    json::write_string(w, &k)?;
                    write!(w, ": ")?;
                }
                write_variable(w, rt, v, EscapeString::Json, tabs)?;
                if i + 1 < n {
                    write!(w, ", ")?;
                }
            }
            write!(w, "}}")?;
        }
        Variable::Array(ref arr) => {
            write!(w, "[")?;
            let n = arr.len();
            for (i, v) in arr.iter().enumerate() {
                write_variable(w, rt, v, EscapeString::Json, tabs)?;
                if i + 1 < n {
                    write!(w, ", ")?;
                }
            }
            write!(w, "]")?;
        }
        Variable::Option(ref opt) => {
            match *opt {
                None => {
                    write!(w, "none()")?
                }
                Some(ref v) => {
                    write!(w, "some(")?;
                    write_variable(w, rt, v, EscapeString::Json, tabs)?;
                    write!(w, ")")?;
                }
            }
        }
        Variable::Result(ref res) => {
            match *res {
                Err(ref err) => {
                    write!(w, "err(")?;
                    write_variable(w, rt, &err.message, EscapeString::Json, tabs)?;
                    write!(w, ")")?;
                }
                Ok(ref ok) => {
                    write!(w, "ok(")?;
                    write_variable(w, rt, ok, EscapeString::Json, tabs)?;
                    write!(w, ")")?;
                }
            }
        }
        Variable::Thread(_) => write!(w, "_thread")?,
        Variable::Return => write!(w, "_return")?,
        Variable::UnsafeRef(_) => write!(w, "_unsafe_ref")?,
        Variable::RustObject(_) => write!(w, "_rust_object")?,
        Variable::Closure(ref closure, _) => write_closure(w, rt, closure, tabs)?,
        Variable::In(_) => write!(w, "_in")?,
        // ref x => panic!("Could not print out `{:?}`", x)
    }
    Ok(())
}

pub(crate) fn print_variable(rt: &Runtime, v: &Variable, escape_string: EscapeString) {
    write_variable(&mut io::stdout(), rt, v, escape_string, 0).unwrap();
}

fn write_tabs<W: io::Write>(w: &mut W, tabs: u32) -> Result<(), io::Error> {
    for _ in 0..tabs {
        write!(w, "    ")?;
    }
    Ok(())
}

fn write_closure<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    closure: &ast::Closure,
    tabs: u32
) -> Result<(), io::Error> {
    write!(w, "\\(")?;
    for (i, arg) in closure.args.iter().enumerate() {
        write_arg(w, arg)?;
        if i + 1 < closure.args.len() {
            write!(w, ", ")?;
        }
    }
    write!(w, ") = ")?;
    write_expr(w, rt, &closure.expr, tabs)?;
    Ok(())
}

fn write_arg<W: io::Write>(
    w: &mut W,
    arg: &ast::Arg
) -> Result<(), io::Error> {
    write!(w, "{}: {}", arg.name, arg.ty.description())
}

fn write_expr<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    expr: &ast::Expression,
    tabs: u32,
) -> Result<(), io::Error> {
    use ast::Expression as E;

    match *expr {
        E::BinOp(ref binop) => write_binop(w, rt, binop.op, &binop.left, &binop.right, tabs)?,
        E::Item(ref item) => write_item(w, rt, item, tabs)?,
        E::Variable(ref range_var) =>
            write_variable(w, rt, &range_var.1, EscapeString::Json, tabs)?,
        E::Link(ref link) => write_link(w, rt, link, tabs)?,
        E::Object(ref obj) => write_obj(w, rt, obj, tabs)?,
        E::Array(ref arr) => write_arr(w, rt, arr, tabs)?,
        E::ArrayFill(ref arr_fill) => write_arr_fill(w, rt, arr_fill, tabs)?,
        E::Call(ref call) => {
            if &**call.name == "norm" && call.args.len() == 1 {
                write_norm(w, rt, &call.args[0], tabs)?
            } else if &**call.name == "not" && call.args.len() == 1 {
                write_not(w, rt, &call.args[0], tabs)?
            } else if &**call.name == "neg" && call.args.len() == 1 {
                write_neg(w, rt, &call.args[0], tabs)?
            } else if &**call.name == "dot" && call.args.len() == 2 {
                write_binop(w, rt, ast::BinOp::Dot, &call.args[0], &call.args[1], tabs)?
            } else if &**call.name == "cross" && call.args.len() == 2 {
                write_binop(w, rt, ast::BinOp::Cross, &call.args[0], &call.args[1], tabs)?
            } else {
                write_call(w, rt, call, tabs)?
            }
        }
        E::Return(ref expr) => {
            write!(w, "return ")?;
            write_expr(w, rt, expr, tabs)?;
        }
        E::ReturnVoid(_) => write!(w, "return")?,
        E::Break(ref br) => {
            if let Some(ref label) = br.label {
                write!(w, "break '{}", label)?;
            } else {
                write!(w, "break")?;
            }
        }
        E::Continue(ref c) => {
            if let Some(ref label) = c.label {
                write!(w, "continue '{}", label)?;
            } else {
                write!(w, "continue")?;
            }
        }
        E::Block(ref b) => write_block(w, rt, b, tabs)?,
        E::Go(ref go) => {
            write!(w, "go ")?;
            write_call(w, rt, &go.call, tabs)?;
        }
        E::Assign(ref assign) => write_assign(w, rt, assign, tabs)?,
        E::Vec4(ref vec4) => write_vec4(w, rt, vec4, tabs)?,
        E::Mat4(ref mat4) => write_mat4(w, rt, mat4, tabs)?,
        E::For(ref f) => write_for(w, rt, f, tabs)?,
        E::Compare(ref comp) => write_compare(w, rt, comp, tabs)?,
        E::ForN(ref for_n) => {
            write!(w, "for ")?;
            write_for_n(w, rt, for_n, tabs)?;
        }
        E::ForIn(ref for_in) => {
            write!(w, "for ")?;
            write_for_in(w, rt, for_in, tabs)?;
        }
        E::Sum(ref for_n) => {
            write!(w, "sum ")?;
            write_for_n(w, rt, for_n, tabs)?;
        }
        E::SumIn(ref for_in) => {
            write!(w, "sum ")?;
            write_for_in(w, rt, for_in, tabs)?;
        }
        E::SumVec4(ref for_n) => {
            write!(w, "sum_vec4 ")?;
            write_for_n(w, rt, for_n, tabs)?;
        }
        E::Prod(ref for_n) => {
            write!(w, "prod ")?;
            write_for_n(w, rt, for_n, tabs)?;
        }
        E::ProdIn(ref for_in) => {
            write!(w, "prod ")?;
            write_for_in(w, rt, for_in, tabs)?;
        }
        E::ProdVec4(ref for_n) => {
            write!(w, "prod_vec4 ")?;
            write_for_n(w, rt, for_n, tabs)?;
        }
        E::Min(ref for_n) => {
            write!(w, "min ")?;
            write_for_n(w, rt, for_n, tabs)?;
        }
        E::MinIn(ref for_in) => {
            write!(w, "min ")?;
            write_for_in(w, rt, for_in, tabs)?;
        }
        E::Max(ref for_n) => {
            write!(w, "max ")?;
            write_for_n(w, rt, for_n, tabs)?;
        }
        E::MaxIn(ref for_in) => {
            write!(w, "max ")?;
            write_for_in(w, rt, for_in, tabs)?;
        }
        E::Sift(ref for_n) => {
            write!(w, "sift ")?;
            write_for_n(w, rt, for_n, tabs)?;
        }
        E::SiftIn(ref for_in) => {
            write!(w, "sift ")?;
            write_for_in(w, rt, for_in, tabs)?;
        }
        E::Any(ref for_n) => {
            write!(w, "any ")?;
            write_for_n(w, rt, for_n, tabs)?;
        }
        E::AnyIn(ref for_in) => {
            write!(w, "any ")?;
            write_for_in(w, rt, for_in, tabs)?;
        }
        E::All(ref for_n) => {
            write!(w, "all ")?;
            write_for_n(w, rt, for_n, tabs)?;
        }
        E::AllIn(ref for_in) => {
            write!(w, "all ")?;
            write_for_in(w, rt, for_in, tabs)?;
        }
        E::LinkFor(ref for_n) => {
            write!(w, "link ")?;
            write_for_n(w, rt, for_n, tabs)?;
        }
        E::LinkIn(ref for_in) => {
            write!(w, "link ")?;
            write_for_in(w, rt, for_in, tabs)?;
        }
        E::If(ref if_expr) => write_if(w, rt, if_expr, tabs)?,
        E::Try(ref expr) => {
            write_expr(w, rt, expr, tabs)?;
            write!(w, "?")?;
        }
        E::Swizzle(ref swizzle) => write_swizzle(w, rt, swizzle, tabs)?,
        E::Closure(ref closure) => write_closure(w, rt, closure, tabs)?,
        E::Grab(ref grab) =>write_grab(w, rt, grab, tabs)?,
        E::TryExpr(ref try_expr) => write_try_expr(w, rt, try_expr, tabs)?,
        E::CallClosure(ref call) => write_call_closure(w, rt, call, tabs)?,
        E::In(ref in_expr) => {
            write!(w, "in {}", in_expr.name)?;
        }
        // x => panic!("Unimplemented `{:#?}`", x),
    }
    Ok(())
}

fn write_norm<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    expr: &ast::Expression,
    tabs: u32
) -> Result<(), io::Error> {
    write!(w, "|")?;
    write_expr(w, rt, &expr, tabs)?;
    write!(w, "|")?;
    Ok(())
}

fn write_block<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    block: &ast::Block,
    tabs: u32,
) -> Result<(), io::Error> {
    match block.expressions.len() {
        0 => {
            write!(w, "{{}}")?;
        }
        1 => {
            write!(w, "{{ ")?;
            write_expr(w, rt, &block.expressions[0], tabs + 1)?;
            write!(w, " }}")?;
        }
        _ => {
            writeln!(w, "{{")?;
            for expr in &block.expressions {
                write_tabs(w, tabs + 1)?;
                write_expr(w, rt, expr, tabs + 1)?;
                writeln!(w, "")?;
            }
            write_tabs(w, tabs)?;
            write!(w, "}}")?;
        }
    }
    Ok(())
}

fn binop_needs_parens(op: ast::BinOp, expr: &ast::Expression, right: bool) -> bool {
    use ast::Expression as E;

    match *expr {
        E::Compare(_) => true,
        E::BinOp(ref binop) => {
            match (op.precedence(), binop.op.precedence()) {
                (3, _) => true,
                (2, 1) => true,
                (2, 2) if right => true,
                (1, 1) if right => true,
                _ => false
            }
        }
        _ => false
    }
}

fn write_binop<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    op: ast::BinOp,
    left: &ast::Expression,
    right: &ast::Expression,
    tabs: u32,
) -> Result<(), io::Error> {
    let left_needs_parens = binop_needs_parens(op, left, false);
    let right_needs_parens = binop_needs_parens(op, right, true);

    if left_needs_parens {
        write!(w, "(")?;
    }
    write_expr(w, rt, left, tabs)?;
    if left_needs_parens {
        write!(w, ")")?;
    }
    write!(w, " {} ", op.symbol())?;
    if right_needs_parens {
        write!(w, "(")?;
    }
    write_expr(w, rt, right, tabs)?;
    if right_needs_parens {
        write!(w, ")")?;
    }
    Ok(())
}

fn write_not<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    expr: &ast::Expression,
    tabs: u32,
) -> Result<(), io::Error> {
    write!(w, "!")?;
    write_expr(w, rt, &expr, tabs)
}

fn write_neg<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    expr: &ast::Expression,
    tabs: u32,
) -> Result<(), io::Error> {
    write!(w, "-")?;
    write_expr(w, rt, &expr, tabs)
}

fn write_item<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    item: &ast::Item,
    tabs: u32,
) -> Result<(), io::Error> {
    use ast::Id;

    if item.current {
        write!(w, "~ ")?;
    }
    write!(w, "{}", item.name)?;
    for (i, id) in item.ids.iter().enumerate() {
        match *id {
            Id::String(_, ref prop) => write!(w, ".{}", prop)?,
            Id::F64(_, ind) => write!(w, "[{}]", ind)?,
            Id::Expression(ref expr) => {
                write!(w, "[")?;
                write_expr(w, rt, expr, tabs)?;
                write!(w, "]")?;
            }
        }
        if item.try_ids.iter().any(|&tr| tr == i) {
            write!(w, "?")?;
        }
    }
    Ok(())
}

fn write_link<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    link: &ast::Link,
    tabs: u32,
) -> Result<(), io::Error> {
    write!(w, "link {{ ")?;
    for item in &link.items {
        write_expr(w, rt, item, tabs)?;
        write!(w, " ")?;
    }
    write!(w, "}}")?;
    Ok(())
}

fn write_obj<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    obj: &ast::Object,
    tabs: u32,
) -> Result<(), io::Error> {
    write!(w, "{{")?;
    for (i, key_value) in obj.key_values.iter().enumerate() {
        if key_value.0.chars().all(|c| c.is_alphanumeric()) {
            write!(w, "{}: ", key_value.0)?;
        } else {
            json::write_string(w, &key_value.0)?;
            write!(w, ": ")?;
        }
        write_expr(w, rt, &key_value.1, tabs + 1)?;
        if i + 1 < obj.key_values.len() {
            write!(w, ", ")?;
        }
    }
    write!(w, "}}")?;
    Ok(())
}

fn write_call<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    call: &ast::Call,
    tabs: u32,
) -> Result<(), io::Error> {
    write!(w, "{}(", call.name)?;
    for (i, arg) in call.args.iter().enumerate() {
        write_expr(w, rt, arg, tabs)?;
        if i + 1 < call.args.len() {
            write!(w, ", ")?;
        }
    }
    write!(w, ")")?;
    Ok(())
}

fn write_call_closure<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    call: &ast::CallClosure,
    tabs: u32,
) -> Result<(), io::Error> {
    write!(w, "\\")?;
    write_item(w, rt, &call.item, tabs)?;
    write!(w, "(")?;
    for (i, arg) in call.args.iter().enumerate() {
        write_expr(w, rt, arg, tabs + 1)?;
        if i + 1 < call.args.len() {
            write!(w, ", ")?;
        }
    }
    write!(w, ")")?;
    Ok(())
}

fn write_arr<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    arr: &ast::Array,
    tabs: u32,
) -> Result<(), io::Error> {
    write!(w, "[")?;
    for (i, item) in arr.items.iter().enumerate() {
        write_expr(w, rt, item, tabs + 1)?;
        if i + 1 < arr.items.len() {
            write!(w, ", ")?;
        }
    }
    write!(w, "]")?;
    Ok(())
}

fn write_arr_fill<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    arr_fill: &ast::ArrayFill,
    tabs: u32,
) -> Result<(), io::Error> {
    write!(w, "[")?;
    write_expr(w, rt, &arr_fill.fill, tabs + 1)?;
    write!(w, ", ")?;
    write_expr(w, rt, &arr_fill.n, tabs + 1)?;
    write!(w, "]")?;
    Ok(())
}

fn write_assign<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    assign: &ast::Assign,
    tabs: u32,
) -> Result<(), io::Error> {
    write_expr(w, rt, &assign.left, tabs)?;
    write!(w, " {} ", assign.op.symbol())?;
    write_expr(w, rt, &assign.right, tabs)?;
    Ok(())
}

fn write_vec4<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    vec4: &ast::Vec4,
    tabs: u32,
) -> Result<(), io::Error> {
    let mut n = vec4.args.len();
    for expr in vec4.args.iter().rev() {
        if let ast::Expression::Variable(ref range_var) = *expr {
            if let (_, Variable::F64(num, _)) = **range_var {
                if num == 0.0 {
                    n -= 1;
                    continue;
                }
            }
        }
        break;
    }
    write!(w, "(")?;
    for (i, expr) in vec4.args[0..n].iter().enumerate() {
        write_expr(w, rt, expr, tabs)?;
        if i + 1 < n {
            write!(w, ", ")?;
        }
        if i + 1 == n && i == 0 {
            write!(w, ",")?;
        }
    }
    write!(w, ")")?;
    Ok(())
}

fn write_mat4<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    mat4: &ast::Mat4,
    tabs: u32,
) -> Result<(), io::Error> {
    let n = mat4.args.len();
    write!(w, "mat4 {{")?;
    for (i, expr) in mat4.args[0..n].iter().enumerate() {
        write_expr(w, rt, expr, tabs)?;
        if i + 1 < n {
            write!(w, "; ")?;
        }
        if i + 1 == n && i == 0 {
            write!(w, ";")?;
        }
    }
    write!(w, "}}")?;
    Ok(())
}

fn write_swizzle<W: io::Write>(
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
    write!(w, "{}", comp(swizzle.sw0))?;
    write!(w, "{}", comp(swizzle.sw1))?;
    if let Some(sw2) = swizzle.sw2 {
        write!(w, "{}", comp(sw2))?;
    }
    if let Some(sw3) = swizzle.sw3 {
        write!(w, "{}", comp(sw3))?;
    }
    write!(w, " ")?;
    write_expr(w, rt, &swizzle.expr, tabs)?;
    Ok(())
}

fn write_for<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    f: &ast::For,
    tabs: u32,
) -> Result<(), io::Error> {
    if let ast::Expression::Block(ref b) = f.init {
        if b.expressions.is_empty() {
            if let ast::Expression::Variable(ref range_var) = f.cond {
                if let (_, Variable::Bool(b, _)) = **range_var {
                    if b {
                        if let ast::Expression::Block(ref b) = f.step {
                            if b.expressions.is_empty() {
                                write!(w, "loop ")?;
                                write_block(w, rt, &f.block, tabs + 1)?;
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }

    write!(w, "for ")?;
    write_expr(w, rt, &f.init, tabs)?;
    write!(w, "; ")?;
    write_expr(w, rt, &f.cond, tabs)?;
    write!(w, "; ")?;
    write_expr(w, rt, &f.step, tabs)?;
    write!(w, " ")?;
    write_block(w, rt, &f.block, tabs + 1)?;
    Ok(())
}

fn compare_needs_parent(expr: &ast::Expression) -> bool {
    use ast::Expression as E;

    match *expr {
        E::BinOp(_) => true,
        _ => false
    }
}

fn write_compare<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    comp: &ast::Compare,
    tabs: u32,
) -> Result<(), io::Error> {
    let left_needs_parens = compare_needs_parent(&comp.left);
    let right_needs_parens = compare_needs_parent(&comp.right);

    if left_needs_parens {
        write!(w, "(")?;
    }
    write_expr(w, rt, &comp.left, tabs)?;
    if left_needs_parens {
        write!(w, ")")?;
    }
    write!(w, " {} ", comp.op.symbol())?;
    if right_needs_parens {
        write!(w, "(")?;
    }
    write_expr(w, rt, &comp.right, tabs)?;
    if right_needs_parens {
        write!(w, ")")?;
    }
    Ok(())
}

fn write_for_n<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    for_n: &ast::ForN,
    tabs: u32
) -> Result<(), io::Error> {
    write!(w, "{} ", for_n.name)?;
    if let Some(ref start) = for_n.start {
        write!(w, "[")?;
        write_expr(w, rt, start, tabs)?;
        write!(w, ", ")?;
        write_expr(w, rt, &for_n.end, tabs)?;
        write!(w, ") ")?;
    } else {
        write_expr(w, rt, &for_n.end, tabs)?;
        write!(w, " ")?;
    }
    write_block(w, rt, &for_n.block, tabs + 1)?;
    Ok(())
}

fn write_for_in<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    for_in: &ast::ForIn,
    tabs: u32
) -> Result<(), io::Error> {
    write!(w, "{} in ", for_in.name)?;
    write_expr(w, rt, &for_in.iter, tabs)?;
    write!(w, " ")?;
    write_block(w, rt, &for_in.block, tabs + 1)?;
    Ok(())
}

fn write_if<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    if_expr: &ast::If,
    tabs: u32,
) -> Result<(), io::Error> {
    write!(w, "if ")?;
    write_expr(w, rt, &if_expr.cond, tabs)?;
    write!(w, " ")?;
    write_block(w, rt, &if_expr.true_block, tabs)?;
    for (else_if_cond, else_if_block) in if_expr.else_if_conds.iter()
        .zip(if_expr.else_if_blocks.iter()) {
        write!(w, " else if ")?;
        write_expr(w, rt, else_if_cond, tabs)?;
        write!(w, " ")?;
        write_block(w, rt, else_if_block, tabs)?;
    }
    if let Some(ref else_block) = if_expr.else_block {
        write!(w, " else ")?;
        write_block(w, rt, else_block, tabs)?;
    }
    Ok(())
}

fn write_grab<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    grab: &ast::Grab,
    tabs: u32,
) -> Result<(), io::Error> {
    if grab.level != 1 {
        write!(w, "(grab '{} ", grab.level)?;
    } else {
        write!(w, "(grab ")?;
    }
    write_expr(w, rt, &grab.expr, tabs)?;
    write!(w, ")")?;
    Ok(())
}

fn write_try_expr<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    try_expr: &ast::TryExpr,
    tabs: u32,
) -> Result<(), io::Error> {
    write!(w, "(try ")?;
    write_expr(w, rt, &try_expr.expr, tabs)?;
    write!(w, ")")?;
    Ok(())
}
