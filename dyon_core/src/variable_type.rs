/// Inherited by traits that need a variable type.
pub trait VariableType<R> {
    /// The type of variable.
    type Variable;
}

impl<T, R, V> VariableType<R> for Option<T>
    where T: VariableType<R, Variable = V>
{
    type Variable = V;
}

impl<T, U, R, V> VariableType<R> for Result<T, U>
    where T: VariableType<R, Variable = V>,
          U: VariableType<R, Variable = V>
{
    type Variable = V;
}

impl<T, R, V> VariableType<R> for [T; 2]
    where T: VariableType<R, Variable = V>
{
    type Variable = V;
}

impl<T, R, V> VariableType<R> for [T; 3]
    where T: VariableType<R, Variable = V>
{
    type Variable = V;
}

impl<T, R, V> VariableType<R> for [T; 4]
    where T: VariableType<R, Variable = V>
{
    type Variable = V;
}

impl<T, R, V> VariableType<R> for Vec<T>
    where T: VariableType<R, Variable = V>
{
    type Variable = V;
}

impl<T, U, R, V> VariableType<R> for (T, U)
    where T: VariableType<R, Variable = V>,
          U: VariableType<R, Variable = V>
{
    type Variable = V;
}

impl<T, U, V, R, Var> VariableType<R> for (T, U, V)
    where T: VariableType<R, Variable = Var>,
          U: VariableType<R, Variable = Var>,
          V: VariableType<R, Variable = Var>
{
    type Variable = Var;
}

impl<T, U, V, W, R, Var> VariableType<R> for (T, U, V, W)
    where T: VariableType<R, Variable = Var>,
          U: VariableType<R, Variable = Var>,
          V: VariableType<R, Variable = Var>,
          W: VariableType<R, Variable = Var>
{
    type Variable = Var;
}
