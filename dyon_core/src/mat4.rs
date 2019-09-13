use super::*;

/// Wraps a 4D matrix for easier embedding with Dyon.
#[derive(Debug, Copy, Clone)]
pub struct Mat4(pub [[f32; 4]; 4]);

/// Convert from and to mat4.
pub trait ConvertMat4: Sized {
    /// Converts mat4 to self.
    fn from(val: [[f32; 4]; 4]) -> Self;
    /// Converts from self to mat4.
    fn to(&self) -> [[f32; 4]; 4];
}

impl ConvertMat4 for Mat4 {
    fn from(val: [[f32; 4]; 4]) -> Self { Mat4(val) }
    fn to(&self) -> [[f32; 4]; 4] { self.0 }
}

impl ConvertMat4 for [[f32; 4]; 4] {
    fn from(val: [[f32; 4]; 4]) -> Self {val}
    fn to(&self) -> [[f32; 4]; 4] {*self}
}

impl<R, V> PopVariable<R> for Mat4
    where Self: VariableType<R, Variable = V>,
          R: RuntimeErrorHandling<Variable = V>,
          V: VariableCore
{
    fn pop_var(rt: &R, var: &V) -> Result<Self, String> {
        if let Some(v) = var.get_mat4() {
            Ok(Mat4(*v))
        } else {
            Err(rt.expected(var, "mat4"))
        }
    }
}

impl<R, V> PushVariable<R> for Mat4
    where Self: VariableType<R, Variable = V>,
          V: VariableCore
{
    fn push_var(&self) -> V { V::mat4(self.0) }
}

impl From<[[f32; 4]; 4]> for Mat4 {
    fn from(val: [[f32; 4]; 4]) -> Mat4 {
        Mat4(val)
    }
}
