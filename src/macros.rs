
#[macro_export]
macro_rules! dyon_macro_items { ($($x:item)+) => ($($x)+) }

#[macro_export]
macro_rules! dyon_fn_pop {
    ($rt:ident $arg:ident : $t:ty) => {
        let $arg: $t = try!($rt.pop());
    };
    ($rt:ident $arg:ident : $t:ty, $($args:tt : $ts:ty),+) => {
        dyon_fn_pop!($rt $($args: $ts),+);
        let $arg: $t = try!($rt.pop());
    };
}

#[macro_export]
macro_rules! dyon_fn {
    (fn $name:ident () -> $rt:ty $b:block) => {
        pub fn $name(rt: &mut $crate::Runtime) -> Result<(), String> {
            fn inner() -> $rt {
                $b
            }

            rt.push(inner());
            Ok(())
        }
    };
    (fn $name:ident ($($arg:tt : $t:ty),+) -> $rt:ty $b:block) => {
        dyon_macro_items!{
            pub fn $name(rt: &mut $crate::Runtime) -> Result<(), String> {
                fn inner($($arg: $t),+) -> $rt {
                    $b
                }

                dyon_fn_pop!(rt $($arg: $t),+);
                rt.push(inner($($arg),+));
                Ok(())
            }
        }
    };
    (fn $name:ident () $b:block) => {
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
                                $f: try!(obj_field(rt, obj, stringify!($f)))
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
