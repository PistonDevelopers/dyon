use std::sync::Arc;

use Object;
use Runtime;
use Variable;

pub fn obj_field<T: PopVariable>(rt: &Runtime, obj: &Object, name: &str) -> Result<T, String> {
    let var = try!(obj.get(&Arc::new(name.into()))
        .ok_or_else(|| format!("Object has no key `{}`", name)));
    PopVariable::pop_var(rt, var)
}

/// Implemented by types that can be popped from the runtime stack.
pub trait PopVariable: Sized {
    /// Converts variable to self.
    /// The variable should be resolved before call.
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String>;
}

/// Implemented by types that can be pushed to the runtime stack.
pub trait PushVariable {
    /// Converts from self to variable.
    fn push_var(&self) -> Variable;
}

/// Implemented by types that can be converted to and from vec4.
pub trait ConvertVec4: Sized {
    /// Converts vec4 to self.
    fn from(val: [f32; 4]) -> Self;
    fn to(&self) -> [f32; 4];
}

impl PopVariable for String {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let &Variable::Text(ref s) = var {
            Ok((&**s).clone())
        } else {
            Err(rt.expected(var, "string"))
        }
    }
}

impl PopVariable for Arc<String> {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let &Variable::Text(ref s) = var {
            Ok(s.clone())
        } else {
            Err(rt.expected(var, "string"))
        }
    }
}

impl PopVariable for u32 {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let &Variable::F64(n) = var {
            Ok(n as u32)
        } else {
            Err(rt.expected(var, "number"))
        }
    }
}

impl PopVariable for f32 {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let &Variable::F64(n) = var {
            Ok(n as f32)
        } else {
            Err(rt.expected(var, "number"))
        }
    }
}

impl PopVariable for f64 {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let &Variable::F64(n) = var {
            Ok(n)
        } else {
            Err(rt.expected(var, "number"))
        }
    }
}

impl<T: PopVariable> PopVariable for Option<T> {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let &Variable::Option(ref s) = var {
            Ok(match *s {
                Some(ref s) => Some(try!(PopVariable::pop_var(rt, rt.resolve(s)))),
                None => None
            })
        } else {
            Err(rt.expected(var, "option"))
        }
    }
}

impl<T: PopVariable> PopVariable for [T; 2] {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let &Variable::Array(ref arr) = var {
            Ok([
                try!(PopVariable::pop_var(rt, rt.resolve(&arr[0]))),
                try!(PopVariable::pop_var(rt, rt.resolve(&arr[1])))
            ])
        } else {
            Err(rt.expected(var, "[_; 2]"))
        }
    }
}

impl<T: PopVariable> PopVariable for [T; 3] {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let &Variable::Array(ref arr) = var {
            Ok([
                try!(PopVariable::pop_var(rt, rt.resolve(&arr[0]))),
                try!(PopVariable::pop_var(rt, rt.resolve(&arr[1]))),
                try!(PopVariable::pop_var(rt, rt.resolve(&arr[2])))
            ])
        } else {
            Err(rt.expected(var, "[_; 3]"))
        }
    }
}

impl<T: PopVariable> PopVariable for [T; 4] {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let &Variable::Array(ref arr) = var {
            Ok([
                try!(PopVariable::pop_var(rt, rt.resolve(&arr[0]))),
                try!(PopVariable::pop_var(rt, rt.resolve(&arr[1]))),
                try!(PopVariable::pop_var(rt, rt.resolve(&arr[2]))),
                try!(PopVariable::pop_var(rt, rt.resolve(&arr[3])))
            ])
        } else {
            Err(rt.expected(var, "[_; 4]"))
        }
    }
}

impl<T: PopVariable> PopVariable for Vec<T> {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let &Variable::Array(ref arr) = var {
            let mut res = Vec::with_capacity(arr.len());
            for it in &**arr {
                res.push(try!(PopVariable::pop_var(rt, rt.resolve(it))))
            }
            Ok(res)
        } else {
            Err(rt.expected(var, "array"))
        }
    }
}

impl PushVariable for bool {
    fn push_var(&self) -> Variable { Variable::Bool(*self) }
}

impl PushVariable for u32 {
    fn push_var(&self) -> Variable { Variable::F64(*self as f64) }
}

impl PushVariable for f32 {
    fn push_var(&self) -> Variable { Variable::F64(*self as f64) }
}

impl PushVariable for f64 {
    fn push_var(&self) -> Variable { Variable::F64(*self) }
}

impl PushVariable for str {
    fn push_var(&self) -> Variable { Variable::Text(Arc::new(self.into())) }
}

impl PushVariable for Arc<String> {
    fn push_var(&self) -> Variable { Variable::Text(self.clone()) }
}

impl<T: PushVariable> PushVariable for Option<T> {
    fn push_var(&self) -> Variable {
        Variable::Option(self.as_ref().map(|v| Box::new(v.push_var())))
    }
}

impl<T: PushVariable> PushVariable for [T; 2] {
    fn push_var(&self) -> Variable {
        Variable::Array(Arc::new(vec![
            self[0].push_var(),
            self[1].push_var()
        ]))
    }
}

impl<T: PushVariable> PushVariable for [T; 3] {
    fn push_var(&self) -> Variable {
        Variable::Array(Arc::new(vec![
            self[0].push_var(),
            self[1].push_var(),
            self[2].push_var()
        ]))
    }
}

impl<T: PushVariable> PushVariable for [T; 4] {
    fn push_var(&self) -> Variable {
        Variable::Array(Arc::new(vec![
            self[0].push_var(),
            self[1].push_var(),
            self[2].push_var(),
            self[3].push_var()
        ]))
    }
}

impl<T: PushVariable> PushVariable for Vec<T> {
    fn push_var(&self) -> Variable {
        Variable::Array(Arc::new(self.iter().map(|it| it.push_var()).collect()))
    }
}

impl ConvertVec4 for [f32; 2] {
    fn from(val: [f32; 4]) -> Self { [val[0], val[1]] }
    fn to(&self) -> [f32; 4] { [self[0], self[1], 0.0, 0.0] }
}

impl ConvertVec4 for [f32; 3] {
    fn from(val: [f32; 4]) -> Self { [val[0], val[1], val[2]] }
    fn to(&self) -> [f32; 4] { [self[0], self[1], self[2], 0.0] }
}

impl ConvertVec4 for [f32; 4] {
    fn from(val: [f32; 4]) -> Self { val }
    fn to(&self) -> [f32; 4] { *self }
}

impl ConvertVec4 for [f64; 2] {
    fn from(val: [f32; 4]) -> Self { [val[0] as f64, val[1] as f64] }
    fn to(&self) -> [f32; 4] { [self[0] as f32, self[1] as f32, 0.0, 0.0] }
}

impl ConvertVec4 for [f64; 3] {
    fn from(val: [f32; 4]) -> Self { [val[0] as f64, val[1] as f64, val[2] as f64] }
    fn to(&self) -> [f32; 4] { [self[0] as f32, self[1] as f32, self[2] as f32, 0.0] }
}

impl ConvertVec4 for [f64; 4] {
    fn from(val: [f32; 4]) -> Self { [val[0] as f64, val[1] as f64, val[2] as f64, val[3] as f64] }
    fn to(&self) -> [f32; 4] { [self[0] as f32, self[1] as f32, self[2] as f32, self[3] as f32] }
}
