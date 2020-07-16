//! Dyon macros.

/// This macro is used by some other Dyon macros.
#[macro_export]
macro_rules! dyon_macro_items { ($($x:item)+) => ($($x)+) }

/// This macro is used by some other Dyon macros.
#[macro_export]
macro_rules! dyon_fn_pop {
    (#&mut $rt:ident) => {};
    (#&mut $rt:ident $arg:ident : $t:ty) => {
        let $arg: RustObject = $rt.pop()?;
        let mut $arg = $arg.lock().map_err(|_| "Could not get a lock on Rust object")?;
        let $arg = $arg.downcast_mut::<$t>()
            .ok_or_else(|| format!("Expected Rust object of type: {}", stringify!($t)))?;
    };
    (#&mut $rt:ident $arg:ident : $t:ty, $($args:tt : $ts:ty),+) => {
        dyon_fn_pop!(#&mut $rt $($args: $ts),+);
        let $arg: RustObject = $rt.pop()?;
        let mut $arg = $arg.lock().map_err(|_| "Could not get a lock on Rust object")?;
        let $arg = $arg.downcast_mut::<$t>()
            .ok_or_else(|| format!("Expected Rust object of type: {}", stringify!($t)))?;
    };
    (#& $rt:ident) => {};
    (#& $rt:ident $arg:ident : $t:ty) => {
        let $arg: RustObject = $rt.pop()?;
        let $arg = $arg.lock().map_err(|_| "Could not get a lock on Rust object")?;
        let $arg = $arg.downcast_ref::<$t>()
            .ok_or_else(|| format!("Expected Rust object of type: {}", stringify!($t)))?;
    };
    (#& $rt:ident $arg:ident : $t:ty, $($args:tt : $ts:ty),+) => {
        dyon_fn_pop!(#& $rt $($args: $ts),+);
        let $arg: RustObject = $rt.pop()?;
        let $arg = $arg.lock().map_err(|_| "Could not get a lock on Rust object")?;
        let $arg = $arg.downcast_ref::<$t>()
            .ok_or_else(|| format!("Expected Rust object of type: {}", stringify!($t)))?;
    };
    (# $rt:ident) => {};
    (# $rt:ident $arg:ident : $t:ty) => {
        let $arg: RustObject = $rt.pop()?;
        let $arg = $arg.lock().map_err(|_| "Could not get a lock on Rust object")?;
        let $arg = *$arg.downcast_ref::<$t>()
            .ok_or_else(|| format!("Expected Rust object of type: {}", stringify!($t)))?;
    };
    (# $rt:ident $arg:ident : $t:ty, $($args:tt : $ts:ty),+) => {
        dyon_fn_pop!(# $rt $($args: $ts),+);
        let $arg: RustObject = $rt.pop()?;
        let $arg = $arg.lock().map_err(|_| "Could not get a lock on Rust object")?;
        let $arg = *$arg.downcast_ref::<$t>()
            .ok_or_else(|| format!("Expected Rust object of type: {}", stringify!($t)))?;
    };
    ($rt:ident) => {};
    ($rt:ident $arg:ident : $t:ty) => {
        let $arg: $t = $rt.pop()?;
    };
    ($rt:ident $arg:ident : $t:ty, $($args:tt : $ts:ty),+) => {
        dyon_fn_pop!($rt $($args: $ts),+);
        let $arg: $t = $rt.pop()?;
    };
}

/// Used to declare an embedded/external function in Rust
/// which can be called from Dyon.
///
/// For example, see "examples/functions.rs".
#[macro_export]
macro_rules! dyon_fn {
    (fn $name:ident () -> # $rt:ty $b:block) => {
        #[allow(non_snake_case)]
        pub fn $name(_rt: &mut $crate::Runtime) -> Result<$crate::Variable, String> {
            use std::sync::{Arc, Mutex};

            fn inner() -> $rt {
                $b
            }

            Ok($crate::Variable::RustObject(Arc::new(Mutex::new(inner())) as RustObject))
        }
    };
    (fn $name:ident ($($rust_arg:tt : #&$rust_t:ty),+) -> # $rt:ty $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(rt: &mut $crate::Runtime) -> Result<$crate::Variable, String> {
                use std::sync::{Arc, Mutex};

                fn inner($($rust_arg: &$rust_t),+) -> $rt {
                    $b
                }

                dyon_fn_pop!(#& rt $($rust_arg: $rust_t),+);
                Ok($crate::Variable::RustObject(Arc::new(Mutex::new(inner($($rust_arg),+)))))
            }
        }
    };
    (fn $name:ident ($rust_arg:tt : #&$rust_t:ty, $($arg:tt : $t:ty),+) -> # $rt:ty $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(rt: &mut $crate::Runtime) -> Result<$crate::Variable, String> {
                use std::sync::{Arc, Mutex};

                fn inner($rust_arg: &$rust_t, $($arg: $t),+) -> $rt {
                    $b
                }

                dyon_fn_pop!(rt $($arg: $t),+);
                dyon_fn_pop!(#& rt $rust_arg: $rust_t);
                Ok($crate::Variable::RustObject(Arc::new(Mutex::new(inner($rust_arg, $($arg),+)))))
            }
        }
    };
    (fn $name:ident ($rust_arg:tt : #&$rust_t:ty , $rust_arg2:tt : #&$rust_t2:ty $(, $arg:tt : $t:ty)*) -> $rt:ty $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(rt: &mut $crate::Runtime) -> Result<$crate::Variable, String> {
                fn inner($rust_arg: &$rust_t, $rust_arg2: &$rust_t2 $(, $arg: $t)*) -> $rt {
                    $b
                }

                dyon_fn_pop!(rt $($arg: $t),*);
                dyon_fn_pop!(#& rt $rust_arg2: $rust_t2);
                dyon_fn_pop!(#& rt $rust_arg: $rust_t);
                Ok($crate::embed::PushVariable::push_var(&inner($rust_arg, $rust_arg2, $($arg),*)))
            }
        }
    };
    (fn $name:ident ($rust_arg:tt : #&$rust_t:ty $(, $arg:tt : $t:ty)*) -> $rt:ty $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(rt: &mut $crate::Runtime) -> Result<$crate::Variable, String> {
                fn inner($rust_arg: &$rust_t $(, $arg: $t)*) -> $rt {
                    $b
                }

                dyon_fn_pop!(rt $($arg: $t),*);
                dyon_fn_pop!(#& rt $rust_arg: $rust_t);
                Ok($crate::embed::PushVariable::push_var(&inner($rust_arg, $($arg),*)))
            }
        }
    };
    (fn $name:ident ($rust_arg:tt : #$rust_t:ty, $($arg:tt : $t:ty),+) -> $rt:ty $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(rt: &mut $crate::Runtime) -> Result<$crate::Variable, String> {
                fn inner($rust_arg: $rust_t, $($arg: $t),+) -> $rt {
                    $b
                }

                dyon_fn_pop!(rt $($arg: $t),+);
                dyon_fn_pop!(# rt $rust_arg: $rust_t);
                Ok($crate::embed::PushVariable::push_var(&inner($rust_arg, $($arg),+)))
            }
        }
    };
    (fn $name:ident ($($arg:tt : $t:ty),*) -> # $rt:ty $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(rt: &mut $crate::Runtime) -> Result<$crate::Variable, String> {
                use std::sync::{Arc, Mutex};

                fn inner($($arg: $t),*) -> $rt {
                    $b
                }

                dyon_fn_pop!(rt $($arg: $t),*);
                Ok($crate::Variable::RustObject(Arc::new(Mutex::new(inner($($arg),*)))))
            }
        }
    };
    (fn $name:ident ($($arg:tt : $t:ty),*) -> $rt:ty $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(_rt: &mut $crate::Runtime) -> Result<$crate::Variable, String> {
                fn inner($($arg: $t),*) -> $rt {
                    $b
                }

                dyon_fn_pop!(_rt $($arg: $t),*);
                Ok($crate::embed::PushVariable::push_var(&inner($($arg),*)))
            }
        }
    };
    (fn $name:ident () $b:block) => {
        #[allow(non_snake_case)]
        pub fn $name(_: &mut $crate::Runtime) -> Result<(), String> {
            fn inner() {
                $b
            }

            inner();
            Ok(())
        }
    };
    (fn $name:ident (
        $rust_arg:tt : #&mut $rust_ty:ty ,
        $rust_arg2:tt : #$rust_ty2:ty
        $(, $arg:tt : $t:ty)*) $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(rt: &mut $crate::Runtime) -> Result<(), String> {
                fn inner($rust_arg: &mut $rust_ty, $rust_arg2: $rust_ty2, $($arg: $t),*) {
                    $b
                }

                dyon_fn_pop!(rt $($arg: $t),*);
                dyon_fn_pop!(# rt $rust_arg2: $rust_ty2);
                dyon_fn_pop!(#&mut rt $rust_arg: $rust_ty);
                inner($rust_arg, $rust_arg2, $($arg),*);
                Ok(())
            }
        }
    };
    (fn $name:ident ($rust_arg:tt : #&mut $rust_ty:ty $(, $arg:tt : $t:ty)*) $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(rt: &mut $crate::Runtime) -> Result<(), String> {
                fn inner($rust_arg: &mut $rust_ty, $($arg: $t),*) {
                    $b
                }

                dyon_fn_pop!(rt $($arg: $t),*);
                dyon_fn_pop!(#&mut rt $rust_arg: $rust_ty);
                inner($rust_arg, $($arg),+);
                Ok(())
            }
        }
    };
    (fn $name:ident ($rust_arg:tt : # $rust_ty:ty , $($arg:tt : $t:ty),*) $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(rt: &mut $crate::Runtime) -> Result<(), String> {
                fn inner($rust_arg: $rust_ty, $($arg: $t),*) {
                    $b
                }

                dyon_fn_pop!(rt $($arg: $t),*);
                dyon_fn_pop!(# rt $rust_arg: $rust_ty);
                inner($rust_arg, $($arg),+);
                Ok(())
            }
        }
    };
    (fn $name:ident ($($arg:tt : #$t:ty),+) $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(rt: &mut $crate::Runtime) -> Result<(), String> {
                fn inner($($arg: $t),+) {
                    $b
                }

                dyon_fn_pop!(# rt $($arg: $t),+);
                inner($($arg),+);
                Ok(())
            }
        }
    };
    (fn $name:ident ($($arg:tt : $t:ty),+) $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(rt: &mut $crate::Runtime) -> Result<(), String> {
                fn inner($($arg: $t),+) {
                    $b
                }

                dyon_fn_pop!(rt $($arg: $t),+);
                inner($($arg),+);
                Ok(())
            }
        }
    };
}

/// Used to implement `embed::PopVariable` and `embed::PushVariable` for some object.
///
/// For example, see "examples/functions.rs".
#[macro_export]
macro_rules! dyon_obj {
    ($t:tt { $($f:tt),* }) => {
        dyon_macro_items!{
            impl $crate::embed::PopVariable for $t {
                fn pop_var(rt: &$crate::Runtime, var: &$crate::Variable) -> Result<Self, String> {
                    use dyon::embed::obj_field;
                    let var = rt.resolve(var);
                    if let &$crate::Variable::Object(ref obj) = var {
                        Ok($t {
                            $(
                                $f: obj_field(rt, obj, stringify!($f))?
                            ),*
                        })
                    } else {
                        Err(rt.expected(var, stringify!($t)))
                    }
                }
            }

            impl $crate::embed::PushVariable for $t {
                fn push_var(&self) -> $crate::Variable {
                    use std::sync::Arc;
                    use std::collections::HashMap;

                    let mut obj: HashMap<_, $crate::Variable> = HashMap::new();
                    $(
                        obj.insert(Arc::new(stringify!($f).into()), self.$f.push_var())
                    ;)*
                    $crate::Variable::Object(Arc::new(obj))
                }
            }
        }
    }
}
