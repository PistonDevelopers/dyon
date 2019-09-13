//! Traits for Dyon interop.

use std::sync::Arc;

pub use dyon_core::{
    PushVariable,
    PopVariable,
    VariableCore,
    VariableType,
    ConvertVec4,
    ConvertMat4,
};

use Object;
use Runtime;
use Variable;
use RustObject;

/// Gets value of object field.
pub fn obj_field<T>(rt: &Runtime, obj: &Object, name: &str) -> Result<T, String>
    where T: PopVariable<Runtime> + VariableType<Runtime, Variable = Variable>
{
    let var = obj.get(&Arc::new(name.into()))
        .ok_or_else(|| format!("Object has no key `{}`", name))?;
    PopVariable::pop_var(rt, var)
}

embed!{Runtime, Variable, RustObject}
