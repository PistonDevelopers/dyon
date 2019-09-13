use std::sync::Arc;

pub use push_variable::*;
pub use pop_variable::*;
pub use variable_type::*;
pub use macros::*;
pub use vec4::*;
pub use mat4::*;

mod push_variable;
mod pop_variable;
mod variable_type;
mod macros;
mod vec4;
mod mat4;

/// A common error message when there is no value on the stack.
pub const TINVOTS: &str = "There is no value on the stack";

pub trait RuntimeEval<Ast, V> {
    fn expression(
        &mut self,
        expr: &Ast,
        side: Side,
    ) -> Result<(Option<V>, Flow), String>;
}

pub trait RuntimeExt<M, V>:
    Sized +
    VariableType<Self, Variable = V> +
    std::ops::DerefMut<Target = RuntimeCore<M, V>> +
    RuntimeResolveReference +
    RuntimeErrorHandling
where V: VariableCore
{
    /// Pops variable from stack.
    fn pop<T>(&mut self) -> Result<T, String>
        where T: PopVariable<Self, Variable = V>
    {
        let v = self.stack.pop().unwrap_or_else(|| panic!(TINVOTS));
        T::pop_var(self, self.resolve(&v))
    }

    /// Pops 4D vector from stack.
    fn pop_vec4<T: ConvertVec4>(&mut self) -> Result<T, String> {
        let v = self.stack.pop().unwrap_or_else(|| panic!(TINVOTS));
        let v = self.resolve(&v);
        if let Some(val) = v.get_vec4() {Ok(T::from(*val))}
        else {Err(self.expected(v, "vec4"))}
    }

    /// Pops 4D matrix from stack.
    fn pop_mat4<T: ConvertMat4>(&mut self) -> Result<T, String> {
        let v = self.stack.pop().unwrap_or_else(|| panic!(TINVOTS));
        let v = self.resolve(&v);
        if let Some(val) = v.get_mat4() {Ok(T::from(*val))}
        else {Err(self.expected(v, "mat4"))}
    }

    /// Gets variable.
    fn var<T>(&self, var: &V) -> Result<T, String>
        where T: PopVariable<Self, Variable = V>
    {
        T::pop_var(self, self.resolve(&var))
    }

    /// Gets Current Object variable from the stack for Current Objects
    /// by finding the corresponding entry in the normal stack.
    /// If the Current Object can't be found in the stack of current objects,
    /// the error ("Could not find current variable `{}`", name) is thrown.
    ///
    /// ##Examples
    ///
    /// Dyon code:
    /// ```text
    /// ~ entity := 42
    /// teleport()
    /// ```
    /// Rust code:
    /// ```ignore
    /// use dyon::{Runtime, RuntimeExt};
    ///
    /// fn teleport(rt: &mut Runtime) -> Result<(), String> {
    ///     let current_entity_id = rt.current_object::<u32>("entity")?;
    ///     assert_eq!(current_entity_id, 42);
    ///     Ok(())
    /// }
    /// ```
    fn current_object<T>(
        &self, name: &str
    ) -> Result<T, String>
        where T: PopVariable<Self, Variable = V>
    {
        let current_object_index = self.current_stack
            .iter()
            .rev()
            .find(|(name_found, _)| **name_found == name)
            .map(|x| x.1)
            .ok_or(format!("Could not find current variable `{}`", name))
            ?;

        T::pop_var(self, self.resolve(&self.stack[current_object_index]))
    }

    /// Gets 4D vector.
    fn var_vec4<T: ConvertVec4>(&self, var: &V) -> Result<T, String> {
        let x = self.resolve(&var);
        if let Some(val) = x.get_vec4() {Ok(T::from(*val))}
        else {Err(self.expected(x, "vec4"))}
    }

    /// Gets 4D matrix.
    fn var_mat4<T: ConvertMat4>(&self, var: &V) -> Result<T, String> {
        let x = self.resolve(&var);
        if let Some(val) = x.get_mat4() {Ok(T::from(*val))}
        else {Err(self.expected(x, "mat4"))}
    }

    /// Push value to stack.
    fn push<T: PushVariable<Self, Variable = V>>(&mut self, val: T) {
        self.stack.push(val.push_var())
    }

    /// Push Vec4 to stack.
    fn push_vec4<T: ConvertVec4>(&mut self, val: T) {
        self.stack.push(V::vec4(val.to()))
    }

    /// Push Mat4 to stack.
    fn push_mat4<T: ConvertMat4>(&mut self, val: T) {
        self.stack.push(V::mat4(val.to()))
    }
}

impl<R, M, V> RuntimeErrorHandling for R
    where R: VariableType<R, Variable = V> +
             std::ops::Deref<Target = RuntimeCore<M, V>>,
          V: VariableCore
{
    fn expected(&self, var: &V, ty: &str) -> String {
        let found_ty = var.typeof_var();
        format!("{}\nExpected `{}`, found `{}`", self.stack_trace(), ty, found_ty)
    }
}

impl<R, M: 'static, V> RuntimeResolveReference for R
    where R: VariableType<R, Variable = V> +
             std::ops::Deref<Target = RuntimeCore<M, V>>,
          V: VariableCore
{
    /// Resolves a variable reference if any, getting a pointer to the variable on the stack.
    #[inline(always)]
    fn resolve<'a>(&'a self, var: &'a V) -> &'a V {
        resolve(&self.stack, var)
    }
}

impl<R, M, V> RuntimeExt<M, V> for R
    where Self:
          Sized +
          VariableType<Self, Variable = V> +
          std::ops::DerefMut<Target = RuntimeCore<M, V>> +
          RuntimeResolveReference +
          RuntimeErrorHandling,
          V: VariableCore
{}

/// Which side an expression is evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// Whether to insert key in object when missing.
    LeftInsert(bool),
    /// Evaluating right side of assignment.
    Right
}

/// Stores return flow, used to continue executing, return, break out of loop or continue loop.
#[derive(Debug)]
pub enum Flow {
    /// Continues execution.
    Continue,
    /// Return from function.
    Return,
    /// Break loop, with optional label.
    Break(Option<Arc<String>>),
    /// Continue loop, with optional label.
    ContinueLoop(Option<Arc<String>>),
}

/// Stores function calls.
#[derive(Debug)]
pub struct Call {
    // was .0
    pub fn_name: Arc<String>,
    /// The index of the relative function in module.
    pub index: usize,
    pub file: Option<Arc<String>>,
    // was .1
    pub stack_len: usize,
    // was .2
    pub local_len: usize,
    pub current_len: usize,
}

/// Stores common fields for a runtime.
pub struct RuntimeCore<M, V> {
    /// Stores current module.
    pub module: Arc<M>,
    /// name, file, stack_len, local_len.
    pub call_stack: Vec<Call>,
    /// Stores stack of locals.
    pub local_stack: Vec<(Arc<String>, usize)>,
    /// Stores stack of current objects.
    ///
    /// When a current object is used, the runtime searches backwards
    /// until it finds the last current variable with the name.
    pub current_stack: Vec<(Arc<String>, usize)>,
    /// Stores variables on the stack.
    pub stack: Vec<V>,
}

impl<M, V> RuntimeCore<M, V> {
    #[inline(always)]
    pub fn push_fn(
        &mut self,
        name: Arc<String>,
        index: usize,
        file: Option<Arc<String>>,
        st: usize,
        lc: usize,
        cu: usize,
    ) {
        self.call_stack.push(Call {
            fn_name: name,
            index,
            file,
            stack_len: st,
            local_len: lc,
            current_len: cu,
        });
    }
    pub fn pop_fn(&mut self, name: Arc<String>) {
        match self.call_stack.pop() {
            None => panic!("Did not call `{}`", name),
            Some(Call { fn_name, stack_len: st, local_len: lc, current_len: cu, .. }) => {
                if name != fn_name {
                    panic!("Calling `{}`, did not call `{}`", fn_name, name);
                }
                self.stack.truncate(st);
                self.local_stack.truncate(lc);
                self.current_stack.truncate(cu);
            }
        }
    }

    pub fn stack_trace(&self) -> String {
        stack_trace(&self.call_stack)
    }
}

pub fn stack_trace(call_stack: &[Call]) -> String {
    let mut s = String::new();
    for call in call_stack.iter() {
        s.push_str(&call.fn_name);
        if let Some(ref file) = call.file {
            s.push_str(" (");
            s.push_str(file);
            s.push(')');
        }
        s.push('\n')
    }
    s
}

#[inline(always)]
pub fn resolve<'a, V>(stack: &'a [V], var: &'a V) -> &'a V
    where V: VariableCore
{
    if let Some(ind) = V::get_ref(var) {
        &stack[ind]
    } else {
        var
    }
}

pub trait RuntimeErrorHandling: Sized + VariableType<Self> {
    fn expected(&self, var: &Self::Variable, name: &str) -> String;
}

pub trait RuntimeResolveReference: Sized + VariableType<Self> {
    fn resolve<'a>(&'a self, var: &'a Self::Variable) -> &'a Self::Variable;
}

pub trait ErrorCore {
    type Variable;
    fn message(&self) -> &Self::Variable;
}

pub trait VariableCore:
    Sized +
    PartialEq +
    std::fmt::Debug +
    Clone +
    Send
{
    type Error: ErrorCore<Variable = Self>;
    type InnerOption: std::ops::Deref<Target = Self>;
    type InnerOk: std::ops::Deref<Target = Self>;
    type InnerErr: std::ops::Deref<Target = Self::Error>;
    type RustObject;
    fn error(self) -> Self::Error;
    fn error_msg(msg: String) -> Self {
        Self::result(Err(Self::error(Self::str(Arc::new(msg)))))
    }
    /// Creates a variable of type `f64`.
    fn f64(val: f64) -> Self;
    /// Creates a variable of type `bool`.
    fn bool(val: bool) -> Self;
    /// Creates a variable of type `str`.
    fn str(val: Arc<String>) -> Self;
    /// Creates a rust object variable.
    fn rust_object(val: Self::RustObject) -> Self;
    fn result(res: Result<Self, Self::Error>) -> Self;
    fn option(val: Option<Self>) -> Self;
    fn mat4(val: [[f32; 4]; 4]) -> Self;
    fn vec4(val: [f32; 4]) -> Self;
    /// Returns type of variable.
    fn typeof_var(&self) -> Arc<String>;
    fn deep_clone(&self, stack: &[Self]) -> Self;
    fn array(arr: Vec<Self>) -> Self;
    fn get_bool(&self) -> Option<bool>;
    fn get_f64(&self) -> Option<f64>;
    fn get_str(&self) -> Option<&Arc<String>>;
    fn get_array(&self) -> Option<&Vec<Self>>;
    fn get_option(&self) -> Option<&Option<Self::InnerOption>>;
    fn get_result(&self) -> Option<&Result<Self::InnerOk, Self::InnerErr>>;
    fn get_rust_object(&self) -> Option<&Self::RustObject>;
    fn get_vec4(&self) -> Option<&[f32; 4]>;
    fn get_mat4(&self) -> Option<&[[f32; 4]; 4]>;
    fn get_ref(&self) -> Option<usize>;
}
