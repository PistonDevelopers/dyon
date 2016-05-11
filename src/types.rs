// use std::collections::HashMap;
use piston_meta::bootstrap::Convert;
use range::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Any,
    Bool,
    F64,
    Vec4,
    String,
    Array(Box<Type>),
    // Object(HashMap<Arc<String>, Type>),
    // Rust(Arc<String>),
    Option(Box<Type>),
    Result(Box<Type>),
}

impl Type {
    pub fn goes_width(&self, other: &Type) -> bool {
        use self::Type::*;

        match self {
            &Any => true,
            &Array(ref arr) => {
                if let &Array(ref other_arr) = other {
                    arr.goes_width(other_arr)
                } else {
                    false
                }
            }
            &Option(ref opt) => {
                if let &Option(ref other_opt) = other {
                    opt.goes_width(other_opt)
                } else {
                    false
                }
            }
            &Result(ref res) => {
                if let &Result(ref other_res) = other {
                    res.goes_width(other_res)
                } else {
                    false
                }
            }
            // Bool, F64, String, Vec4.
            x if x == other => { true }
            _ => false
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
                ty = Some(Type::String);
            } else if let Ok((range, _)) = convert.meta_bool("vec4") {
                convert.update(range);
                ty = Some(Type::Vec4);
            } else if let Ok((range, _)) = convert.meta_bool("opt_any") {
                convert.update(range);
                ty = Some(Type::Option(Box::new(Type::Any)));
            } else if let Ok((range, _)) = convert.meta_bool("res_any") {
                convert.update(range);
                ty = Some(Type::Result(Box::new(Type::Any)));
            } else if let Ok((range, val)) = Type::from_meta_data(
                    "opt", convert, ignored) {
                convert.update(range);
                ty = Some(Type::Option(Box::new(val)));
            } else if let Ok((range, val)) = Type::from_meta_data(
                    "res", convert, ignored) {
                convert.update(range);
                ty = Some(Type::Result(Box::new(val)));
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), try!(ty.ok_or(()))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
