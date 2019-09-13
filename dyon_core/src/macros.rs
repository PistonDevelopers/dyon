
#[macro_export]
macro_rules! fn_external_ref {
    ($Runtime:ty) => {
        /// Used to store direct reference to external function.
        #[derive(Copy)]
        pub struct FnExternalRef(pub fn(&mut $Runtime) -> Result<(), String>);

        impl Clone for FnExternalRef {
            fn clone(&self) -> FnExternalRef {
                *self
            }
        }

        impl fmt::Debug for FnExternalRef {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "FnExternalRef")
            }
        }
    }
}

#[macro_export]
macro_rules! fn_index {
    () => {
        /// Refers to a function.
        #[derive(Clone, Copy, Debug)]
        pub enum FnIndex {
            /// No function.
            None,
            /// Relative to function you call from.
            Loaded(isize),
            /// External function with no return value.
            ExternalVoid(FnExternalRef),
            /// Extern function with return value.
            ExternalReturn(FnExternalRef),
        }
    }
}

#[macro_export]
macro_rules! fn_external {
    ($Runtime:ty) => {
        struct FnExternal {
            namespace: Arc<Vec<Arc<String>>>,
            name: Arc<String>,
            f: fn(&mut $Runtime) -> Result<(), String>,
            p: Dfn,
        }

        impl Clone for FnExternal {
            fn clone(&self) -> FnExternal {
                FnExternal {
                    namespace: self.namespace.clone(),
                    name: self.name.clone(),
                    f: self.f,
                    p: self.p.clone(),
                }
            }
        }
    }
}

#[macro_export]
macro_rules! unsafe_ref {
    ($Variable:ty) => {
        /// Prevents unsafe references from being accessed outside library.
        #[derive(Debug, Clone)]
        pub struct UnsafeRef(*mut $Variable);
    }
}

#[macro_export]
macro_rules! variable_type_impl {
    ($Runtime:ty, $Variable:ty, $T:ty) => {
        impl $crate::VariableType<$Runtime> for $T {type Variable = $Variable;}
    }
}

#[macro_export]
macro_rules! variable_type {
    ($Runtime:ty, $Variable:ty, $RustObject:ty) => {
        variable_type_impl!{$Runtime, $Variable, $Runtime}
        variable_type_impl!{$Runtime, $Variable, $Variable}
        variable_type_impl!{$Runtime, $Variable, $RustObject}
        variable_type_impl!{$Runtime, $Variable, bool}
        variable_type_impl!{$Runtime, $Variable, u32}
        variable_type_impl!{$Runtime, $Variable, usize}
        variable_type_impl!{$Runtime, $Variable, f32}
        variable_type_impl!{$Runtime, $Variable, f64}
        variable_type_impl!{$Runtime, $Variable, str}
        variable_type_impl!{$Runtime, $Variable, String}
        variable_type_impl!{$Runtime, $Variable, std::sync::Arc<String>}
        variable_type_impl!{$Runtime, $Variable, $crate::Vec4}
        variable_type_impl!{$Runtime, $Variable, $crate::Mat4}
    }
}

#[macro_export]
macro_rules! pop_variable {
    ($Runtime:ty, $Variable:ty) => {
        impl PopVariable<$Runtime> for Variable {
            fn pop_var(rt: &$Runtime, var: &$Variable) -> Result<Self, String> {
                Ok(var.deep_clone(&rt.stack))
            }
        }
    }
}

#[macro_export]
macro_rules! pop_bool {
    ($Runtime:ty, $Variable:ty) => {
        impl PopVariable<$Runtime> for bool {
            fn pop_var(rt: &$Runtime, var: &$Variable) -> Result<Self, String> {
                use $crate::RuntimeErrorHandling;
                if let Some(b) = var.get_bool() {
                    Ok(b)
                } else {
                    Err(rt.expected(var, "bool"))
                }
            }
        }
    }
}

#[macro_export]
macro_rules! pop_u32 {
    ($Runtime:ty, $Variable:ty) => {
        impl PopVariable<$Runtime> for u32 {
            fn pop_var(rt: &$Runtime, var: &$Variable) -> Result<Self, String> {
                use $crate::RuntimeErrorHandling;
                if let Some(n) = var.get_f64() {
                    Ok(n as u32)
                } else {
                    Err(rt.expected(var, "number"))
                }
            }
        }
    }
}

#[macro_export]
macro_rules! pop_usize {
    ($Runtime:ty, $Variable:ty) => {
        impl PopVariable<$Runtime> for usize {
            fn pop_var(rt: &$Runtime, var: &$Variable) -> Result<Self, String> {
                use $crate::RuntimeErrorHandling;
                if let Some(n) = var.get_f64() {
                    Ok(n as usize)
                } else {
                    Err(rt.expected(var, "number"))
                }
            }
        }
    }
}

#[macro_export]
macro_rules! pop_f32 {
    ($Runtime:ty, $Variable:ty) => {
        impl PopVariable<$Runtime> for f32 {
            fn pop_var(rt: &$Runtime, var: &$Variable) -> Result<Self, String> {
                use $crate::RuntimeErrorHandling;
                if let Some(n) = var.get_f64() {
                    Ok(n as f32)
                } else {
                    Err(rt.expected(var, "number"))
                }
            }
        }
    }
}

#[macro_export]
macro_rules! pop_f64 {
    ($Runtime:ty, $Variable:ty) => {
        impl PopVariable<$Runtime> for f64 {
            fn pop_var(rt: &$Runtime, var: &$Variable) -> Result<Self, String> {
                use $crate::RuntimeErrorHandling;
                if let Some(n) = var.get_f64() {
                    Ok(n)
                } else {
                    Err(rt.expected(var, "number"))
                }
            }
        }
    }
}

#[macro_export]
macro_rules! pop_str {
    ($Runtime:ty, $Variable:ty) => {
        impl PopVariable<$Runtime> for Arc<String> {
            fn pop_var(rt: &$Runtime, var: &$Variable) -> Result<Self, String> {
                use $crate::RuntimeErrorHandling;
                if let Some(s) = var.get_str() {
                    Ok(s.clone())
                } else {
                    Err(rt.expected(var, "string"))
                }
            }
        }

        impl PopVariable<$Runtime> for String {
            fn pop_var(rt: &$Runtime, var: &$Variable) -> Result<Self, String> {
                use $crate::RuntimeErrorHandling;
                if let Some(s) = var.get_str() {
                    Ok((&**s).clone())
                } else {
                    Err(rt.expected(var, "string"))
                }
            }
        }
    }
}

#[macro_export]
macro_rules! pop_rust_object {
    ($Runtime:ty, $Variable:ty, $RustObject:ty) => {
        impl PopVariable<$Runtime> for $RustObject {
            fn pop_var(rt: &$Runtime, var: &$Variable) -> Result<Self, String> {
                use $crate::RuntimeErrorHandling;
                if let Some(robj) = var.get_rust_object() {
                    Ok(robj.clone())
                } else {
                    Err(rt.expected(var, "rust_object"))
                }
            }
        }
    }
}

#[macro_export]
macro_rules! pop {
    ($Runtime:ty, $Variable:ty, $RustObject:ty) => {
        pop_variable!{$Runtime, $Variable}
        pop_bool!{$Runtime, $Variable}
        pop_u32!{$Runtime, $Variable}
        pop_usize!{$Runtime, $Variable}
        pop_f32!{$Runtime, $Variable}
        pop_f64!{$Runtime, $Variable}
        pop_str!{$Runtime, $Variable}
        pop_rust_object!{$Runtime, $Variable, $RustObject}
    }
}

#[macro_export]
macro_rules! push_variable {
    ($Runtime:ty, $Variable:ty) => {
        impl PushVariable<$Runtime> for $Variable {
            fn push_var(&self) -> $Variable { self.clone() }
        }
    }
}

#[macro_export]
macro_rules! push_rust_object {
    ($Runtime:ty, $Variable:ty, $RustObject:ty) => {
        impl PushVariable<$Runtime> for $RustObject {
            fn push_var(&self) -> $Variable {
                <$Variable>::rust_object(self.clone())
            }
        }
    }
}

#[macro_export]
macro_rules! push_bool {
    ($Runtime:ty, $Variable:ty) => {
        impl PushVariable<$Runtime> for bool {
            fn push_var(&self) -> $Variable { <$Variable>::bool(*self) }
        }
    }
}

#[macro_export]
macro_rules! push_u32 {
    ($Runtime:ty, $Variable:ty) => {
        impl PushVariable<$Runtime> for u32 {
            fn push_var(&self) -> $Variable { <$Variable>::f64(f64::from(*self)) }
        }
    }
}

#[macro_export]
macro_rules! push_usize {
    ($Runtime:ty, $Variable:ty) => {
        impl PushVariable<$Runtime> for usize {
            fn push_var(&self) -> $Variable { <$Variable>::f64(*self as f64) }
        }
    }
}

#[macro_export]
macro_rules! push_f32 {
    ($Runtime:ty, $Variable:ty) => {
        impl PushVariable<$Runtime> for f32 {
            fn push_var(&self) -> $Variable { <$Variable>::f64(f64::from(*self)) }
        }
    }
}

#[macro_export]
macro_rules! push_f64 {
    ($Runtime:ty, $Variable:ty) => {
        impl PushVariable<$Runtime> for f64 {
            fn push_var(&self) -> $Variable { <$Variable>::f64(*self) }
        }
    }
}

#[macro_export]
macro_rules! push_str {
    ($Runtime:ty, $Variable:ty) => {
        impl PushVariable<$Runtime> for str {
            fn push_var(&self) -> $Variable { <$Variable>::str(std::sync::Arc::new(self.into())) }
        }

        impl PushVariable<$Runtime> for String {
            fn push_var(&self) -> $Variable { <$Variable>::str(std::sync::Arc::new(self.clone())) }
        }

        impl PushVariable<$Runtime> for Arc<String> {
            fn push_var(&self) -> $Variable { <$Variable>::str(self.clone()) }
        }
    }
}

#[macro_export]
macro_rules! push {
    ($Runtime:ty, $Variable:ty, $RustObject:ty) => {
        push_variable!{$Runtime, $Variable}
        push_rust_object!{$Runtime, $Variable, $RustObject}
        push_bool!{$Runtime, $Variable}
        push_u32!{$Runtime, $Variable}
        push_usize!{$Runtime, $Variable}
        push_f32!{$Runtime, $Variable}
        push_f64!{$Runtime, $Variable}
        push_str!{$Runtime, $Variable}
    }
}

#[macro_export]
macro_rules! embed {
    ($Runtime:ty, $Variable:ty, $RustObject:ty) => {
        variable_type!{$Runtime, $Variable, $RustObject}
        pop!{$Runtime, $Variable, $RustObject}
        push!{$Runtime, $Variable, $RustObject}
    }
}
