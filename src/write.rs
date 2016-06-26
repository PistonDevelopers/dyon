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
    escape_string: EscapeString
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
            try!(write_variable(w, rt, &rt.stack[ind], escape_string));
        }
        Variable::Link(ref link) => {
            match escape_string {
                EscapeString::Json => {
                    // Write link items.
                    try!(write!(w, "link {{ "));
                    for slice in &link.slices {
                        for i in slice.start..slice.end {
                            let v = slice.block.var(i);
                            try!(write_variable(w, rt, &v, EscapeString::Json));
                            try!(write!(w, " "));
                        }
                    }
                    try!(write!(w, "}}"));
                }
                EscapeString::None => {
                    for slice in &link.slices {
                        for i in slice.start..slice.end {
                            let v = slice.block.var(i);
                            try!(write_variable(w, rt, &v, EscapeString::None));
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
                try!(write_variable(w, rt, v, EscapeString::Json));
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
                try!(write_variable(w, rt, v, EscapeString::Json));
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
                    try!(write_variable(w, rt, v, EscapeString::Json));
                    try!(write!(w, ")"));
                }
            }
        }
        Variable::Result(ref res) => {
            match res {
                &Err(ref err) => {
                    try!(write!(w, "err("));
                    try!(write_variable(w, rt, &err.message,
                                        EscapeString::Json));
                    try!(write!(w, ")"));
                }
                &Ok(ref ok) => {
                    try!(write!(w, "ok("));
                    try!(write_variable(w, rt, ok, EscapeString::Json));
                    try!(write!(w, ")"));
                }
            }
        }
        Variable::Thread(_) => try!(write!(w, "_thread")),
        Variable::Return => try!(write!(w, "_return")),
        Variable::UnsafeRef(_) => try!(write!(w, "_unsafe_ref")),
        Variable::RustObject(_) => try!(write!(w, "_rust_object")),
        Variable::Closure(ref closure, _) => try!(write_closure(w, rt, closure)),
        // ref x => panic!("Could not print out `{:?}`", x)
    }
    Ok(())
}

pub fn print_variable(rt: &Runtime, v: &Variable, escape_string: EscapeString) {
    write_variable(&mut io::stdout(), rt, v, escape_string).unwrap();
}

pub fn write_closure<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    closure: &ast::Closure
) -> Result<(), io::Error> {
    try!(write!(w, "\\("));
    for (i, arg) in closure.args.iter().enumerate() {
        try!(write_arg(w, arg));
        if i + 1 < closure.args.len() {
            try!(write!(w, ", "));
        }
    }
    try!(write!(w, ") = "));
    try!(write_expr(w, rt, &closure.expr));
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
    expr: &ast::Expression
) -> Result<(), io::Error> {
    use ast::Expression as E;

    match expr {
        &E::BinOp(ref binop) => try!(write_binop(w, rt, binop)),
        &E::Item(ref item) => try!(write_item(w, rt, item)),
        &E::Number(ref number) => try!(write!(w, "{}", number.num)),
        &E::Variable(_, ref var) => try!(write_variable(w, rt, var, EscapeString::Json)),
        x => panic!("Unimplemented `{:#?}`", x),
    }
    Ok(())
}

pub fn write_binop<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    binop: &ast::BinOpExpression
) -> Result<(), io::Error> {
    try!(write_expr(w, rt, &binop.left));
    try!(write!(w, " {} ", binop.op.symbol()));
    try!(write_expr(w, rt, &binop.right));
    Ok(())
}

pub fn write_item<W: io::Write>(
    w: &mut W,
    rt: &Runtime,
    item: &ast::Item
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
            &Id::Expression(ref expr) => try!(write_expr(w, rt, expr)),
        }
        if item.try_ids.iter().any(|&tr| tr == i) {
            try!(write!(w, "?"));
        }
    }
    Ok(())
}
