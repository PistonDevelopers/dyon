//! Traits for Dyon interop.

use std::sync::Arc;

use crate::{
    Error,
    Object,
    Runtime,
    RustObject,
    Variable,
};

/// Gets value of object field.
pub fn obj_field<T: PopVariable>(rt: &Runtime, obj: &Object, name: &str) -> Result<T, String> {
    let var = obj
        .get(&Arc::new(name.into()))
        .ok_or_else(|| format!("Object has no key `{}`", name))?;
    PopVariable::pop_var(rt, var)
}

/// Implemented by types that can be popped from the runtime stack.
pub trait PopVariable: Sized {
    /// Converts variable to self.
    /// The variable should be looked up before call.
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
    /// Converts from self to mat4.
    fn to(&self) -> [f32; 4];
}

/// Convert from and to mat4.
pub trait ConvertMat4: Sized {
    /// Converts mat4 to self.
    fn from(val: [[f32; 4]; 4]) -> Self;
    /// Converts from self to mat4.
    fn to(&self) -> [[f32; 4]; 4];
}

impl PopVariable for Variable {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        Ok(var.deep_clone(&rt.stack))
    }
}

impl PopVariable for RustObject {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::RustObject(ref robj) = *var {
            Ok(robj.clone())
        } else {
            Err(rt.expected(var, "rust_object"))
        }
    }
}

impl PopVariable for bool {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Bool(b, _) = *var {
            Ok(b)
        } else {
            Err(rt.expected(var, "bool"))
        }
    }
}

impl PopVariable for String {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Str(ref s) = *var {
            Ok((&**s).clone())
        } else {
            Err(rt.expected(var, "string"))
        }
    }
}

impl PopVariable for Arc<String> {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Str(ref s) = *var {
            Ok(s.clone())
        } else {
            Err(rt.expected(var, "string"))
        }
    }
}

impl PopVariable for u32 {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::F64(n, _) = *var {
            Ok(n as u32)
        } else {
            Err(rt.expected(var, "number"))
        }
    }
}

impl PopVariable for usize {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::F64(n, _) = *var {
            Ok(n as usize)
        } else {
            Err(rt.expected(var, "number"))
        }
    }
}

impl PopVariable for f32 {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::F64(n, _) = *var {
            Ok(n as f32)
        } else {
            Err(rt.expected(var, "number"))
        }
    }
}

impl PopVariable for f64 {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::F64(n, _) = *var {
            Ok(n)
        } else {
            Err(rt.expected(var, "number"))
        }
    }
}

impl<T: PopVariable> PopVariable for Option<T> {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Option(ref s) = *var {
            Ok(match *s {
                Some(ref s) => Some(PopVariable::pop_var(rt, rt.get(s))?),
                None => None,
            })
        } else {
            Err(rt.expected(var, "option"))
        }
    }
}

impl<T: PopVariable, U: PopVariable> PopVariable for Result<T, U> {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Result(ref s) = *var {
            Ok(match *s {
                Ok(ref s) => Ok(PopVariable::pop_var(rt, rt.get(s))?),
                Err(ref err) => Err(PopVariable::pop_var(rt, rt.get(&err.message))?),
            })
        } else {
            Err(rt.expected(var, "result"))
        }
    }
}

impl<T: PopVariable> PopVariable for [T; 2] {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Array(ref arr) = *var {
            Ok([
                PopVariable::pop_var(rt, rt.get(&arr[0]))?,
                PopVariable::pop_var(rt, rt.get(&arr[1]))?,
            ])
        } else {
            Err(rt.expected(var, "[_; 2]"))
        }
    }
}

impl<T: PopVariable> PopVariable for [T; 3] {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Array(ref arr) = *var {
            Ok([
                PopVariable::pop_var(rt, rt.get(&arr[0]))?,
                PopVariable::pop_var(rt, rt.get(&arr[1]))?,
                PopVariable::pop_var(rt, rt.get(&arr[2]))?,
            ])
        } else {
            Err(rt.expected(var, "[_; 3]"))
        }
    }
}

impl<T: PopVariable> PopVariable for [T; 4] {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Array(ref arr) = *var {
            Ok([
                PopVariable::pop_var(rt, rt.get(&arr[0]))?,
                PopVariable::pop_var(rt, rt.get(&arr[1]))?,
                PopVariable::pop_var(rt, rt.get(&arr[2]))?,
                PopVariable::pop_var(rt, rt.get(&arr[3]))?,
            ])
        } else {
            Err(rt.expected(var, "[_; 4]"))
        }
    }
}

impl<T: PopVariable, U: PopVariable> PopVariable for (T, U) {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Array(ref arr) = *var {
            Ok((
                PopVariable::pop_var(rt, rt.get(&arr[0]))?,
                PopVariable::pop_var(rt, rt.get(&arr[1]))?,
            ))
        } else {
            Err(rt.expected(var, "[_; 2]"))
        }
    }
}

impl<T: PopVariable, U: PopVariable, V: PopVariable> PopVariable for (T, U, V) {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Array(ref arr) = *var {
            Ok((
                PopVariable::pop_var(rt, rt.get(&arr[0]))?,
                PopVariable::pop_var(rt, rt.get(&arr[1]))?,
                PopVariable::pop_var(rt, rt.get(&arr[2]))?,
            ))
        } else {
            Err(rt.expected(var, "[_; 3]"))
        }
    }
}

impl<T: PopVariable, U: PopVariable, V: PopVariable, W: PopVariable> PopVariable for (T, U, V, W) {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Array(ref arr) = *var {
            Ok((
                PopVariable::pop_var(rt, rt.get(&arr[0]))?,
                PopVariable::pop_var(rt, rt.get(&arr[1]))?,
                PopVariable::pop_var(rt, rt.get(&arr[2]))?,
                PopVariable::pop_var(rt, rt.get(&arr[3]))?,
            ))
        } else {
            Err(rt.expected(var, "[_; 4]"))
        }
    }
}

impl<T: PopVariable> PopVariable for Vec<T> {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Array(ref arr) = *var {
            let mut res = Vec::with_capacity(arr.len());
            for it in &**arr {
                res.push(PopVariable::pop_var(rt, rt.get(it))?)
            }
            Ok(res)
        } else {
            Err(rt.expected(var, "array"))
        }
    }
}

impl PushVariable for Variable {
    fn push_var(&self) -> Variable {
        self.clone()
    }
}

impl PushVariable for RustObject {
    fn push_var(&self) -> Variable {
        Variable::RustObject(self.clone())
    }
}

impl PushVariable for bool {
    fn push_var(&self) -> Variable {
        Variable::bool(*self)
    }
}

impl PushVariable for u32 {
    fn push_var(&self) -> Variable {
        Variable::f64(f64::from(*self))
    }
}

impl PushVariable for usize {
    fn push_var(&self) -> Variable {
        Variable::f64(*self as f64)
    }
}

impl PushVariable for f32 {
    fn push_var(&self) -> Variable {
        Variable::f64(f64::from(*self))
    }
}

impl PushVariable for f64 {
    fn push_var(&self) -> Variable {
        Variable::f64(*self)
    }
}

impl PushVariable for str {
    fn push_var(&self) -> Variable {
        Variable::Str(Arc::new(self.into()))
    }
}

impl PushVariable for String {
    fn push_var(&self) -> Variable {
        Variable::Str(Arc::new(self.clone()))
    }
}

impl PushVariable for Arc<String> {
    fn push_var(&self) -> Variable {
        Variable::Str(self.clone())
    }
}

impl<T: PushVariable> PushVariable for Option<T> {
    fn push_var(&self) -> Variable {
        Variable::Option(self.as_ref().map(|v| Box::new(v.push_var())))
    }
}

impl<T: PushVariable, U: PushVariable> PushVariable for Result<T, U> {
    fn push_var(&self) -> Variable {
        Variable::Result(self.as_ref().map(|v| Box::new(v.push_var())).map_err(|e| {
            Box::new(Error {
                message: e.push_var(),
                trace: vec![],
            })
        }))
    }
}

impl<T: PushVariable> PushVariable for [T; 2] {
    fn push_var(&self) -> Variable {
        Variable::Array(Arc::new(vec![self[0].push_var(), self[1].push_var()]))
    }
}

impl<T: PushVariable> PushVariable for [T; 3] {
    fn push_var(&self) -> Variable {
        Variable::Array(Arc::new(vec![
            self[0].push_var(),
            self[1].push_var(),
            self[2].push_var(),
        ]))
    }
}

impl<T: PushVariable> PushVariable for [T; 4] {
    fn push_var(&self) -> Variable {
        Variable::Array(Arc::new(vec![
            self[0].push_var(),
            self[1].push_var(),
            self[2].push_var(),
            self[3].push_var(),
        ]))
    }
}

impl<T: PushVariable, U: PushVariable> PushVariable for (T, U) {
    fn push_var(&self) -> Variable {
        Variable::Array(Arc::new(vec![self.0.push_var(), self.1.push_var()]))
    }
}

impl<T: PushVariable, U: PushVariable, V: PushVariable> PushVariable for (T, U, V) {
    fn push_var(&self) -> Variable {
        Variable::Array(Arc::new(vec![
            self.0.push_var(),
            self.1.push_var(),
            self.2.push_var(),
        ]))
    }
}

impl<T: PushVariable, U: PushVariable, V: PushVariable, W: PushVariable> PushVariable
    for (T, U, V, W)
{
    fn push_var(&self) -> Variable {
        Variable::Array(Arc::new(vec![
            self.0.push_var(),
            self.1.push_var(),
            self.2.push_var(),
            self.3.push_var(),
        ]))
    }
}

impl<T: PushVariable> PushVariable for Vec<T> {
    fn push_var(&self) -> Variable {
        Variable::Array(Arc::new(self.iter().map(|it| it.push_var()).collect()))
    }
}

impl ConvertVec4 for [f32; 2] {
    fn from(val: [f32; 4]) -> Self {
        [val[0], val[1]]
    }
    fn to(&self) -> [f32; 4] {
        [self[0], self[1], 0.0, 0.0]
    }
}

impl ConvertVec4 for [f32; 3] {
    fn from(val: [f32; 4]) -> Self {
        [val[0], val[1], val[2]]
    }
    fn to(&self) -> [f32; 4] {
        [self[0], self[1], self[2], 0.0]
    }
}

impl ConvertVec4 for [f32; 4] {
    fn from(val: [f32; 4]) -> Self {
        val
    }
    fn to(&self) -> [f32; 4] {
        *self
    }
}

impl ConvertVec4 for [f64; 2] {
    fn from(val: [f32; 4]) -> Self {
        [f64::from(val[0]), f64::from(val[1])]
    }
    fn to(&self) -> [f32; 4] {
        [self[0] as f32, self[1] as f32, 0.0, 0.0]
    }
}

impl ConvertVec4 for [f64; 3] {
    fn from(val: [f32; 4]) -> Self {
        [f64::from(val[0]), f64::from(val[1]), f64::from(val[2])]
    }
    fn to(&self) -> [f32; 4] {
        [self[0] as f32, self[1] as f32, self[2] as f32, 0.0]
    }
}

impl ConvertVec4 for [f64; 4] {
    fn from(val: [f32; 4]) -> Self {
        [
            f64::from(val[0]),
            f64::from(val[1]),
            f64::from(val[2]),
            f64::from(val[3]),
        ]
    }
    fn to(&self) -> [f32; 4] {
        [
            self[0] as f32,
            self[1] as f32,
            self[2] as f32,
            self[3] as f32,
        ]
    }
}

impl ConvertMat4 for [[f32; 4]; 4] {
    fn from(val: [[f32; 4]; 4]) -> Self {
        val
    }
    fn to(&self) -> [[f32; 4]; 4] {
        *self
    }
}
