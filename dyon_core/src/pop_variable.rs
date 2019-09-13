use super::*;

/// Implemented by types that can be popped from the runtime stack.
pub trait PopVariable<R>: Sized + VariableType<R> {
    /// Converts variable to self.
    /// The variable should be resolved before call.
    fn pop_var(rt: &R, var: &Self::Variable) -> Result<Self, String>;
}

impl<T, R, V> PopVariable<R> for Option<T>
    where T: PopVariable<R, Variable = V>,
          R: RuntimeResolveReference<Variable = V> +
             RuntimeErrorHandling<Variable = V>,
          V: VariableCore
{
    fn pop_var(rt: &R, var: &V) -> Result<Self, String> {
        if let Some(s) = var.get_option() {
            Ok(match *s {
                Some(ref s) => Some(PopVariable::pop_var(rt, rt.resolve(s))?),
                None => None
            })
        } else {
            Err(rt.expected(var, "option"))
        }
    }
}

impl<T, U, R, V> PopVariable<R> for Result<T, U>
    where T: PopVariable<R, Variable = V>,
          U: PopVariable<R, Variable = V>,
          R: RuntimeResolveReference<Variable = V> +
             RuntimeErrorHandling<Variable = V>,
          V: VariableCore
{
    fn pop_var(rt: &R, var: &V) -> Result<Self, String> {
        if let Some(ref s) = var.get_result() {
            Ok(match *s {
                Ok(ref s) => Ok(PopVariable::pop_var(rt, rt.resolve(s))?),
                Err(ref err) => Err(PopVariable::pop_var(rt, rt.resolve(err.message()))?)
            })
        } else {
            Err(rt.expected(var, "result"))
        }
    }
}

impl<T, R, V> PopVariable<R> for [T; 2]
    where T: PopVariable<R, Variable = V>,
          R: RuntimeResolveReference<Variable = V> +
             RuntimeErrorHandling<Variable = V>,
          V: VariableCore
{
    fn pop_var(rt: &R, var: &V) -> Result<Self, String> {
        if let Some(ref arr) = var.get_array() {
            Ok([
                PopVariable::pop_var(rt, rt.resolve(&arr[0]))?,
                PopVariable::pop_var(rt, rt.resolve(&arr[1]))?
            ])
        } else {
            Err(rt.expected(var, "[_; 2]"))
        }
    }
}

impl<T, R, V> PopVariable<R> for [T; 3]
    where T: PopVariable<R, Variable = V>,
          R: RuntimeResolveReference<Variable = V> +
             RuntimeErrorHandling<Variable = V>,
          V: VariableCore
{
    fn pop_var(rt: &R, var: &V) -> Result<Self, String> {
        if let Some(arr) = var.get_array() {
            Ok([
                PopVariable::pop_var(rt, rt.resolve(&arr[0]))?,
                PopVariable::pop_var(rt, rt.resolve(&arr[1]))?,
                PopVariable::pop_var(rt, rt.resolve(&arr[2]))?
            ])
        } else {
            Err(rt.expected(var, "[_; 3]"))
        }
    }
}

impl<T, R, V> PopVariable<R> for [T; 4]
    where T: PopVariable<R, Variable = V>,
          R: RuntimeResolveReference<Variable = V> +
             RuntimeErrorHandling<Variable = V>,
          V: VariableCore
{
    fn pop_var(rt: &R, var: &V) -> Result<Self, String> {
        if let Some(arr) = var.get_array() {
            Ok([
                PopVariable::pop_var(rt, rt.resolve(&arr[0]))?,
                PopVariable::pop_var(rt, rt.resolve(&arr[1]))?,
                PopVariable::pop_var(rt, rt.resolve(&arr[2]))?,
                PopVariable::pop_var(rt, rt.resolve(&arr[3]))?
            ])
        } else {
            Err(rt.expected(var, "[_; 4]"))
        }
    }
}

impl<T, U, R, V> PopVariable<R> for (T, U)
    where T: PopVariable<R, Variable = V>,
          U: PopVariable<R, Variable = V>,
          R: RuntimeResolveReference<Variable = V> +
             RuntimeErrorHandling<Variable = V>,
          V: VariableCore
{
    fn pop_var(rt: &R, var: &V) -> Result<Self, String> {
        if let Some(ref arr) = var.get_array() {
            Ok((
                PopVariable::pop_var(rt, rt.resolve(&arr[0]))?,
                PopVariable::pop_var(rt, rt.resolve(&arr[1]))?
            ))
        } else {
            Err(rt.expected(var, "[_; 2]"))
        }
    }
}

impl<T, U, V, R, Var> PopVariable<R> for (T, U, V)
    where T: PopVariable<R, Variable = Var>,
          U: PopVariable<R, Variable = Var>,
          V: PopVariable<R, Variable = Var>,
          R: RuntimeResolveReference<Variable = Var> +
             RuntimeErrorHandling<Variable = Var>,
          Var: VariableCore
{
    fn pop_var(rt: &R, var: &Var) -> Result<Self, String> {
        if let Some(ref arr) = var.get_array() {
            Ok((
                PopVariable::pop_var(rt, rt.resolve(&arr[0]))?,
                PopVariable::pop_var(rt, rt.resolve(&arr[1]))?,
                PopVariable::pop_var(rt, rt.resolve(&arr[2]))?
            ))
        } else {
            Err(rt.expected(var, "[_; 3]"))
        }
    }
}

impl<T, U, V, W, R, Var> PopVariable<R> for (T, U, V, W)
    where T: PopVariable<R, Variable = Var>,
          U: PopVariable<R, Variable = Var>,
          V: PopVariable<R, Variable = Var>,
          W: PopVariable<R, Variable = Var>,
          R: RuntimeResolveReference<Variable = Var> +
             RuntimeErrorHandling<Variable = Var>,
          Var: VariableCore
{
    fn pop_var(rt: &R, var: &Var) -> Result<Self, String> {
        if let Some(ref arr) = var.get_array() {
            Ok((
                PopVariable::pop_var(rt, rt.resolve(&arr[0]))?,
                PopVariable::pop_var(rt, rt.resolve(&arr[1]))?,
                PopVariable::pop_var(rt, rt.resolve(&arr[2]))?,
                PopVariable::pop_var(rt, rt.resolve(&arr[3]))?
            ))
        } else {
            Err(rt.expected(var, "[_; 4]"))
        }
    }
}

impl<T, R, V> PopVariable<R> for Vec<T>
    where T: PopVariable<R, Variable = V>,
          R: RuntimeResolveReference<Variable = V> +
             RuntimeErrorHandling<Variable = V>,
          V: VariableCore
{
    fn pop_var(rt: &R, var: &V) -> Result<Self, String> {
        if let Some(arr) = var.get_array() {
            let mut res = Vec::with_capacity(arr.len());
            for it in &**arr {
                res.push(PopVariable::pop_var(rt, rt.resolve(it))?)
            }
            Ok(res)
        } else {
            Err(rt.expected(var, "array"))
        }
    }
}
