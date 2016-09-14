use std::sync::Arc;

use piston_meta::bootstrap::Convert;
use range::Range;
use Dfn;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Whether a statement is never reached.
    Unreachable,
    Void,
    Any,
    Bool,
    F64,
    Vec4,
    Text,
    Link,
    Array(Box<Type>),
    // Object(HashMap<Arc<String>, Type>),
    Object,
    // Rust(Arc<String>),
    Option(Box<Type>),
    Result(Box<Type>),
    Secret(Box<Type>),
    Thread(Box<Type>),
    AdHoc(Arc<String>, Box<Type>),
    Closure(Box<Dfn>),
}

impl Type {
    pub fn description(&self) -> String {
        use Type::*;

        match self {
            &Unreachable => "unreachable".into(),
            &Void => "void".into(),
            &Any => "any".into(),
            &Bool => "bool".into(),
            &F64 => "f64".into(),
            &Vec4 => "vec4".into(),
            &Text => "str".into(),
            &Link => "link".into(),
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
            &Secret(ref ty) => {
                match **ty {
                    Bool => "sec[bool]".into(),
                    F64 => "sec[f64]".into(),
                    _ => panic!("Secret only supports `bool` and `f64`")
                }
            }
            &Thread(ref ty) => {
                if let Any = **ty {
                    "thr".into()
                } else {
                    let mut res = String::from("thr[");
                    res.push_str(&ty.description());
                    res.push(']');
                    res
                }
            }
            &AdHoc(ref ad, ref ty) => {
                (&**ad).clone() + " " + &ty.description()
            }
            &Closure(ref closure) => {
                let mut s = String::new();
                s.push_str("\\(");
                for (i, ty) in closure.tys.iter().enumerate() {
                    s.push_str(&ty.description());
                    if i + 1 < closure.tys.len() {
                        s.push_str(", ");
                    }
                }
                s.push_str(") -> ");
                s.push_str(&closure.ret.description());
                s
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

    pub fn thread() -> Type {
        Type::Thread(Box::new(Type::Any))
    }

    /// Returns `true` if a type goes with another type (directional check).
    ///
    /// - `bool` (argument) goes with `sec[bool]` (value)
    /// - `f64` (argument) goes with `sec[f64]` (value)
    ///
    /// The opposite is not true, since `sec` contains extra information.
    pub fn goes_with(&self, other: &Type) -> bool {
        use self::Type::*;

        // Invert the order because of complex ad-hoc logic.
        if let &AdHoc(_, _) = other {
            if let &AdHoc(_, _) = self {}
            else {
                return other.goes_with(self)
            }
        }
        if let &Secret(ref other_ty) = other {
            return if let &Secret(ref this_ty) = self {
                    this_ty.goes_with(other_ty)
                } else {
                    self.goes_with(other_ty)
                };
        }
        match self {
            // Unreachable goes with anything.
            &Unreachable => true,
            _ if *other == Unreachable => true,
            &Any => *other != Void,
            // Void only goes with void.
            &Void => *other == Void,
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
            &Thread(ref thr) => {
                if let &Thread(ref other_thr) = other {
                    thr.goes_with(other_thr)
                } else if let &Any = other {
                    true
                } else {
                    false
                }
            }
            &Closure(ref cl) => {
                if let &Closure(ref other_cl) = other {
                    if cl.tys.len() != other_cl.tys.len() { return false; }
                    if !cl.tys.iter().zip(other_cl.tys.iter()).all(|(a, b)| a.goes_with(b)) {
                        return false;
                    }
                    if !cl.ret.goes_with(&other_cl.ret) { return false; }
                    true
                } else if let &Any = other {
                    true
                } else {
                    false
                }
            }
            &AdHoc(ref name, ref ty) => {
                if let &AdHoc(ref other_name, ref other_ty) = other {
                    name == other_name && ty.goes_with(other_ty)
                } else if let &Void = other {
                    false
                } else {
                    ty.goes_with(other)
                }
            }
            // Bool, F64, Text, Vec4, AdHoc.
            x if x == other => { true }
            _ if *other == Type::Any => { true }
            _ => { false }
        }
    }

    pub fn add(&self, other: &Type) -> Option<Type> {
        use self::Type::*;

        match (self, other) {
            (&AdHoc(ref name, ref ty), &AdHoc(ref other_name, ref other_ty)) => {
                if name != other_name { return None; }
                if !ty.goes_with(other_ty) { return None; }
                if let Some(new_ty) = ty.add(other_ty) {
                    Some(AdHoc(name.clone(), Box::new(new_ty)))
                } else {
                    None
                }
            }
            (&Void, _) | (_, &Void) => None,
            (&Array(_), _) | (_, &Array(_)) => None,
            (&Bool, &Bool) => Some(Bool),
            (&Secret(ref a), &Secret(ref b))
            if **a == Type::Bool && **b == Type::Bool =>
                Some(Secret(Box::new(Bool))),
            (&Secret(ref a), &Bool) if **a == Type::Bool => Some(Secret(Box::new(Bool))),
            (&Bool, &Secret(ref b)) if **b == Type::Bool => Some(Bool),
            (&F64, &F64) => Some(F64),
            (&Secret(ref a), &Secret(ref b))
            if **a == Type::F64 && **b == Type::F64 =>
                Some(Secret(Box::new(F64))),
            (&Secret(ref a), &F64) if **a == Type::F64 => Some(Secret(Box::new(F64))),
            (&F64, &Secret(ref b)) if **b == Type::F64 => Some(F64),
            (&Text, &Text) => Some(Text),
            (&Vec4, &F64) => Some(Vec4),
            (&F64, &Vec4) => Some(Vec4),
            (&Vec4, &Vec4) => Some(Vec4),
            (&Any, x) if x != &Type::Void => Some(Any),
            (x, &Any) if x != &Type::Void => Some(Any),
            _ => None
        }
    }

    pub fn add_assign(&self, other: &Type) -> bool {
        use self::Type::*;

        match (self, other) {
            (&AdHoc(ref name, ref ty), &AdHoc(ref other_name, ref other_ty)) => {
                if name != other_name { return false; }
                if !ty.goes_with(other_ty) { return false; }
                ty.add_assign(other_ty)
            }
            (&AdHoc(_, _), _) | (_, &AdHoc(_, _)) => false,
            (&Void, _) | (_, &Void) => false,
            _ => true
        }
    }

    pub fn mul(&self, other: &Type) -> Option<Type> {
        use self::Type::*;

        match (self, other) {
            (&Void, _) | (_, &Void) => None,
            (&Array(_), _) | (_, &Array(_)) => None,
            (&Bool, &Bool) => Some(Bool),
            (&Secret(ref a), &Secret(ref b))
            if **a == Type::Bool && **b == Type::Bool =>
                Some(Secret(Box::new(Bool))),
            (&Secret(ref a), &Bool) if **a == Type::Bool => Some(Secret(Box::new(Bool))),
            (&Bool, &Secret(ref b)) if **b == Type::Bool => Some(Bool),
            (&F64, &F64) => Some(F64),
            (&Secret(ref a), &Secret(ref b))
            if **a == Type::F64 && **b == Type::F64 =>
                Some(Secret(Box::new(F64))),
            (&Secret(ref a), &F64) if **a == Type::F64 => Some(Secret(Box::new(F64))),
            (&F64, &Secret(ref b)) if **b == Type::F64 => Some(F64),
            (&Vec4, &F64) => Some(Vec4),
            (&F64, &Vec4) => Some(Vec4),
            (&Vec4, &Vec4) => Some(Vec4),
            (&Any, x) if x != &Type::Void => Some(Any),
            (x, &Any) if x != &Type::Void => Some(Any),
            _ => None
        }
    }

    pub fn pow(&self, other: &Type) -> Option<Type> {
        use self::Type::*;

        match (self, other) {
            (&Void, _) | (_, &Void) => None,
            (&Array(_), _) | (_, &Array(_)) => None,
            (&Bool, &Bool) => Some(Bool),
            (&Secret(ref a), &Secret(ref b))
            if **a == Type::Bool && **b == Type::Bool =>
                Some(Secret(Box::new(Bool))),
            (&Secret(ref a), &Bool) if **a == Type::Bool => Some(Secret(Box::new(Bool))),
            (&Bool, &Secret(ref b)) if **b == Type::Bool => Some(Bool),
            (&F64, &F64) => Some(F64),
            (&Secret(ref a), &Secret(ref b))
            if **a == Type::F64 && **b == Type::F64 =>
                Some(Secret(Box::new(F64))),
            (&Secret(ref a), &F64) if **a == Type::F64 => Some(Secret(Box::new(F64))),
            (&F64, &Secret(ref b)) if **b == Type::F64 => Some(F64),
            (&Vec4, &F64) => Some(Vec4),
            (&F64, &Vec4) => Some(Vec4),
            (&Vec4, &Vec4) => Some(Vec4),
            (&Any, x) if x != &Type::Void => Some(Any),
            (x, &Any) if x != &Type::Void => Some(Any),
            _ => None
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
            } else if let Ok((range, _)) = convert.meta_bool("any") {
                convert.update(range);
                ty = Some(Type::Any);
            } else if let Ok((range, _)) = convert.meta_bool("bool") {
                convert.update(range);
                ty = Some(Type::Bool);
            } else if let Ok((range, _)) = convert.meta_bool("sec_bool") {
                convert.update(range);
                ty = Some(Type::Secret(Box::new(Type::Bool)));
            } else if let Ok((range, _)) = convert.meta_bool("f64") {
                convert.update(range);
                ty = Some(Type::F64);
            } else if let Ok((range, _)) = convert.meta_bool("sec_f64") {
                convert.update(range);
                ty = Some(Type::Secret(Box::new(Type::F64)));
            } else if let Ok((range, _)) = convert.meta_bool("str") {
                convert.update(range);
                ty = Some(Type::Text);
            } else if let Ok((range, _)) = convert.meta_bool("vec4") {
                convert.update(range);
                ty = Some(Type::Vec4);
            } else if let Ok((range, _)) = convert.meta_bool("link") {
                convert.update(range);
                ty = Some(Type::Link);
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
            } else if let Ok((range, _)) = convert.meta_bool("thr_any") {
                convert.update(range);
                ty = Some(Type::Thread(Box::new(Type::Any)));
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
            } else if let Ok((range, val)) = Type::from_meta_data(
                    "thr", convert, ignored) {
                convert.update(range);
                ty = Some(Type::Thread(Box::new(val)));
            } else if let Ok((range, val)) = convert.meta_string("ad_hoc") {
                convert.update(range);
                let inner_ty = if let Ok((range, val)) = Type::from_meta_data(
                        "ad_hoc_ty", convert, ignored) {
                    convert.update(range);
                    val
                } else {
                    Type::Object
                };
                ty = Some(Type::AdHoc(val, Box::new(inner_ty)));
            } else if let Ok(range) = convert.start_node("closure_type") {
                convert.update(range);
                let mut lts = vec![];
                let mut tys = vec![];
                while let Ok((range, val)) = Type::from_meta_data(
                        "cl_arg", convert, ignored) {
                    use Lt;

                    convert.update(range);
                    lts.push(Lt::Default);
                    tys.push(val);
                }
                let (range, ret) = try!(Type::from_meta_data("cl_ret", convert, ignored));
                convert.update(range);
                let range = try!(convert.end_node("closure_type"));
                convert.update(range);
                ty = Some(Type::Closure(Box::new(Dfn { lts: lts, tys: tys, ret: ret })));
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), try!(ty.ok_or(()))))
    }
}
