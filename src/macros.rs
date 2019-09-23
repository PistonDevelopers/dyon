//! Dyon macros.

/// This macro is used by some other Dyon macros.
#[macro_export]
macro_rules! dyon_macro_items { ($($x:item)+) => ($($x)+) }

/// This macro is used by some other Dyon macros.
#[macro_export]
macro_rules! dyon_fn_pop {
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
    (fn $name:ident () -> $rt:ty $b:block) => {
        #[allow(non_snake_case)]
        pub fn $name(_rt: &mut $crate::Runtime) -> Result<$crate::Variable, String> {
            fn inner() -> $rt {
                $b
            }

            Ok($crate::embed::PushVariable::push_var(&inner()))
        }
    };
    (fn $name:ident ($($arg:tt : $t:ty),+) -> $rt:ty $b:block) => {
        dyon_macro_items!{
            #[allow(non_snake_case)]
            pub fn $name(rt: &mut $crate::Runtime) -> Result<$crate::Variable, String> {
                fn inner($($arg: $t),+) -> $rt {
                    $b
                }

                dyon_fn_pop!(rt $($arg: $t),+);
                Ok($crate::embed::PushVariable::push_var(&inner($($arg),+)))
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
