use crate::embed::{ConvertMat4, PopVariable, PushVariable};
use crate::{Runtime, Variable};

/// Wraps a 4D matrix for easier embedding with Dyon.
#[derive(Debug, Copy, Clone)]
pub struct Mat4(pub [[f32; 4]; 4]);

impl ConvertMat4 for Mat4 {
    fn from(val: [[f32; 4]; 4]) -> Self {
        Mat4(val)
    }
    fn to(&self) -> [[f32; 4]; 4] {
        self.0
    }
}

impl PopVariable for Mat4 {
    fn pop_var(rt: &Runtime, var: &Variable) -> Result<Self, String> {
        if let Variable::Mat4(ref v) = *var {
            Ok(Mat4(**v))
        } else {
            Err(rt.expected(var, "mat4"))
        }
    }
}

impl PushVariable for Mat4 {
    fn push_var(&self) -> Variable {
        Variable::Mat4(Box::new(self.0))
    }
}

impl From<[[f32; 4]; 4]> for Mat4 {
    fn from(val: [[f32; 4]; 4]) -> Mat4 {
        Mat4(val)
    }
}
