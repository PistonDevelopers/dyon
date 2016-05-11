// use std::collections::HashMap;
use piston_meta::bootstrap::Convert;
use range::Range;

use ast;
use Module;
use Prelude;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Any,
    Bool,
    F64,
    Vec4,
    Text,
    Array(Box<Type>),
    // Object(HashMap<Arc<String>, Type>),
    Object,
    // Rust(Arc<String>),
    Option(Box<Type>),
    Result(Box<Type>),
}

impl Type {
    pub fn description(&self) -> String {
        use Type::*;

        match self {
            &Void => "void".into(),
            &Any => "any".into(),
            &Bool => "bool".into(),
            &F64 => "f64".into(),
            &Vec4 => "vec4".into(),
            &Text => "str".into(),
            &Array(ref ty) => {
                if let Any = **ty {
                    "[]".into()
                } else {
                    let mut res = String::from("[");
                    res.push_str(&ty.description());
                    res.push(']');
                    res
                }
            }
            &Object => "{}".into(),
            &Option(ref ty) => {
                if let Any = **ty {
                    "opt".into()
                } else {
                    let mut res = String::from("opt[");
                    res.push_str(&ty.description());
                    res.push(']');
                    res
                }
            }
            &Result(ref ty) => {
                if let Any = **ty {
                    "res".into()
                } else {
                    let mut res = String::from("res[");
                    res.push_str(&ty.description());
                    res.push(']');
                    res
                }
            }
        }
    }

    pub fn array() -> Type {
        Type::Array(Box::new(Type::Any))
    }

    pub fn object() -> Type {
        Type::Object
    }

    pub fn option() -> Type {
        Type::Option(Box::new(Type::Any))
    }

    pub fn result() -> Type {
        Type::Result(Box::new(Type::Any))
    }

    pub fn goes_with(&self, other: &Type) -> bool {
        use self::Type::*;

        match self {
            &Any => *other != Type::Void,
            &Array(ref arr) => {
                if let &Array(ref other_arr) = other {
                    arr.goes_with(other_arr)
                } else if let &Any = other {
                    true
                } else {
                    false
                }
            }
            &Object => {
                if let &Object = other {
                    true
                } else if let &Any = other {
                    true
                } else {
                    false
                }
            }
            &Option(ref opt) => {
                if let &Option(ref other_opt) = other {
                    opt.goes_with(other_opt)
                } else if let &Any = other {
                    true
                } else {
                    false
                }
            }
            &Result(ref res) => {
                if let &Result(ref other_res) = other {
                    res.goes_with(other_res)
                } else if let &Any = other {
                    true
                } else {
                    false
                }
            }
            // Void, Bool, F64, Text, Vec4.
            x if x == other => { true }
            _ if *other == Type::Any => { true }
            _ => { false }
        }
    }

    pub fn from_meta_data(node: &str, mut convert: Convert, ignored: &mut Vec<Range>)
    -> Result<(Range, Type), ()> {
        let start = convert.clone();
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut ty: Option<Type> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, _)) = convert.meta_bool("bool") {
                convert.update(range);
                ty = Some(Type::Bool);
            } else if let Ok((range, _)) = convert.meta_bool("f64") {
                convert.update(range);
                ty = Some(Type::F64);
            } else if let Ok((range, _)) = convert.meta_bool("str") {
                convert.update(range);
                ty = Some(Type::Text);
            } else if let Ok((range, _)) = convert.meta_bool("vec4") {
                convert.update(range);
                ty = Some(Type::Vec4);
            } else if let Ok((range, _)) = convert.meta_bool("opt_any") {
                convert.update(range);
                ty = Some(Type::Option(Box::new(Type::Any)));
            } else if let Ok((range, _)) = convert.meta_bool("res_any") {
                convert.update(range);
                ty = Some(Type::Result(Box::new(Type::Any)));
            } else if let Ok((range, _)) = convert.meta_bool("arr_any") {
                convert.update(range);
                ty = Some(Type::Array(Box::new(Type::Any)));
            } else if let Ok((range, _)) = convert.meta_bool("obj_any") {
                convert.update(range);
                ty = Some(Type::Object);
            } else if let Ok((range, val)) = Type::from_meta_data(
                    "opt", convert, ignored) {
                convert.update(range);
                ty = Some(Type::Option(Box::new(val)));
            } else if let Ok((range, val)) = Type::from_meta_data(
                    "res", convert, ignored) {
                convert.update(range);
                ty = Some(Type::Result(Box::new(val)));
            } else if let Ok((range, val)) = Type::from_meta_data(
                    "arr", convert, ignored) {
                convert.update(range);
                ty = Some(Type::Array(Box::new(val)));
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), try!(ty.ok_or(()))))
    }
}

fn check_call(call: &ast::Call, module: &Module, prelude: &Prelude) -> Result<Type, String> {
    if let Some(f) = module.functions.get(&call.name) {
        if f.args.len() != call.args.len() {
            return Err(module.error(call.source_range,
                &format!("Type mismatch: Expected {} arguments, found {}", f.args.len(), call.args.len())));
        }
        for (i, arg) in call.args.iter().enumerate() {
            let ty = try!(check_expr(arg, module, prelude));
            if !ty.goes_with(&f.args[i].ty) {
                return Err(module.error(arg.source_range(),
                    &format!("Type mismatch: Expected `{}`, found `{}`",
                        f.args[i].ty.description(), ty.description())))
            }
        }
        Ok(f.ret.clone())
    } else if let Some(f) = prelude.functions.get(&call.name) {
        if f.tys.len() != call.args.len() {
            return Err(module.error(call.source_range,
                &format!("Type mismatch: Expected {} arguments, found {}", f.tys.len(), call.args.len())));
        }
        for (i, arg) in call.args.iter().enumerate() {
            let ty = try!(check_expr(arg, module, prelude));
            if !ty.goes_with(&f.tys[i]) {
                return Err(module.error(arg.source_range(),
                    &format!("Type mismatch: Expected `{}`, found `{}`",
                        f.tys[i].description(), ty.description())))
            }
        }
        Ok(f.ret.clone())
    } else {
        return Err(module.error(call.source_range,
            &format!("TYPECHK: Could not find declaration of `{}`", call.name)))
    }
}

fn check_expr(expr: &ast::Expression, module: &Module, prelude: &Prelude) -> Result<Type, String> {
    use ast::Expression::*;

    match expr {
        &Call(ref call) => check_call(call, module, prelude),
        &Bool(_) => Ok(Type::Bool),
        // TODO: Get type from declaration.
        &Item(_) => Ok(Type::Any),
        &Object(_) => Ok(Type::Any),
        &Array(_) => Ok(Type::array()),
        &ArrayFill(_) => Ok(Type::array()),
        &Return(ref expr) => check_expr(expr, module, prelude),
        &ReturnVoid(_) => Ok(Type::Void),
        &Break(_) => Ok(Type::Void),
        &Continue(_) => Ok(Type::Void),
        &Block(ref block) => check_block(block, module, prelude),
        &BinOp(ref _binop) => Ok(Type::Any),
        &Assign(ref _assign) => Ok(Type::Void),
        &Text(_) => Ok(Type::Text),
        &Number(_) => Ok(Type::F64),
        &Vec4(_) => Ok(Type::Vec4),
        &For(_) => Ok(Type::Void),
        &ForN(_) => Ok(Type::Void),
        &Sum(_) => Ok(Type::F64),
        &Min(_) => Ok(Type::F64),
        &Max(_) => Ok(Type::F64),
        &Sift(_) => Ok(Type::array()),
        &Any(_) => Ok(Type::Bool),
        &All(_) => Ok(Type::Bool),
        &If(ref if_expr) => {
            let ty = try!(check_expr(&if_expr.cond, module, prelude));
            if !ty.goes_with(&Type::Bool) {
                return Err(module.error(if_expr.cond.source_range(),
                    &format!("Type mismatch: Expected `{}`, found `{}`",
                        Type::Bool.description(), ty.description())))
            }
            Ok(Type::Any)
        }
        &Compare(_) => Ok(Type::Bool),
        &UnOp(_) => Ok(Type::Any),
        &Variable(_, _) => Ok(Type::Any),
        &Try(_) => Ok(Type::Any),
    }
}

fn check_block(block: &ast::Block, module: &Module, prelude: &Prelude) -> Result<Type, String> {
    let mut ty: Option<Type> = None;
    for expr in &block.expressions {
        ty = Some(try!(check_expr(expr, module, prelude)));
    }
    Ok(ty.unwrap_or(Type::Void))
}

pub fn run(module: &Module, prelude: &Prelude) -> Result<(), String> {
    for f in module.functions.values() {
        try!(check_block(&f.block, module, prelude));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // use super::*;
}
