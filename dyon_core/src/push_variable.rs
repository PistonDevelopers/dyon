use super::*;

/// Implemented by types that can be pushed to the runtime stack.
pub trait PushVariable<R>: VariableType<R> {
    /// Converts from self to variable.
    fn push_var(&self) -> Self::Variable;
}

impl<T, R, V> PushVariable<R> for Option<T>
    where T: PushVariable<R, Variable = V>,
          V: VariableCore
{
    fn push_var(&self) -> V {
        V::option(self.as_ref().map(|v| v.push_var()))
    }
}

impl<T, U, R, V> PushVariable<R> for Result<T, U>
    where T: PushVariable<R, Variable = V>,
          U: PushVariable<R, Variable = V>,
          V: VariableCore
{
    fn push_var(&self) -> V {
        V::result(self.as_ref()
            .map(|v| v.push_var())
            .map_err(|e| e.push_var().error()))
    }
}

impl<T, R, V> PushVariable<R> for [T; 2]
    where T: PushVariable<R, Variable = V>,
          V: VariableCore
{
    fn push_var(&self) -> V {
        V::array(vec![
            self[0].push_var(),
            self[1].push_var()
        ])
    }
}

impl<T, R, V> PushVariable<R> for [T; 3]
    where T: PushVariable<R, Variable = V>,
          V: VariableCore
{
    fn push_var(&self) -> V {
        V::array(vec![
            self[0].push_var(),
            self[1].push_var(),
            self[2].push_var()
        ])
    }
}

impl<T, R, V> PushVariable<R> for [T; 4]
    where T: PushVariable<R, Variable = V>,
          V: VariableCore
{
    fn push_var(&self) -> V {
        V::array(vec![
            self[0].push_var(),
            self[1].push_var(),
            self[2].push_var(),
            self[3].push_var()
        ])
    }
}

impl<T, U: PushVariable<R>, R, V> PushVariable<R> for (T, U)
    where T: PushVariable<R, Variable = V>,
          U: PushVariable<R, Variable = V>,
          V: VariableCore
{
    fn push_var(&self) -> V {
        V::array(vec![
            self.0.push_var(),
            self.1.push_var()
        ])
    }
}

impl<T, U, V: PushVariable<R>, R, Var> PushVariable<R> for (T, U, V)
    where T: PushVariable<R, Variable = Var>,
          U: PushVariable<R, Variable = Var>,
          V: PushVariable<R, Variable = Var>,
          Var: VariableCore
{
    fn push_var(&self) -> Var {
        Var::array(vec![
            self.0.push_var(),
            self.1.push_var(),
            self.2.push_var()
        ])
    }
}

impl<T, U, V, W, R, Var>
PushVariable<R> for (T, U, V, W)
    where T: PushVariable<R, Variable = Var>,
          U: PushVariable<R, Variable = Var>,
          V: PushVariable<R, Variable = Var>,
          W: PushVariable<R, Variable = Var>,
          Var: VariableCore
{
    fn push_var(&self) -> Var {
        Var::array(vec![
            self.0.push_var(),
            self.1.push_var(),
            self.2.push_var(),
            self.3.push_var()
        ])
    }
}

impl<T, R, V: VariableCore> PushVariable<R> for Vec<T>
    where T: PushVariable<R, Variable = V>,
          Vec<V>: std::iter::FromIterator<T::Variable>
{
    fn push_var(&self) -> V {
        V::array(self.iter().map(|it| it.push_var()).collect())
    }
}
