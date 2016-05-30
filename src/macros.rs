
#[macro_export]
macro_rules! dyon_macro_items { ($($x:item)+) => ($($x)+) }

#[macro_export]
macro_rules! dyon_fn {
    (fn $name:ident () -> $rt:ty $b:block) => {
        fn $name(rt: &mut $crate::Runtime) -> Result<(), String> {
            rt.push::<$rt>($b);
            Ok(())
        }
    };
    (fn $name:ident ($arg:tt : $t:ty) -> $rt:ty $b:block) => {
        dyon_macro_items!{
            fn $name(rt: &mut $crate::Runtime) -> Result<(), String> {
                let $arg: $t = try!(rt.pop());
                rt.push::<$rt>($b);
                Ok(())
            }
        }
    };
    (fn $name:ident ($arg0:tt : $t0:ty, $arg1:tt : $t1:ty) -> $rt:ty $b:block) => {
        dyon_macro_items!{
            fn $name(rt: &mut $crate::Runtime) -> Result<(), String> {
                let $arg1: $t1 = try!(rt.pop());
                let $arg0: $t0 = try!(rt.pop());
                rt.push::<$rt>($b);
                Ok(())
            }
        }
    };
    (fn $name:ident () $b:block) => {
        fn $name(_: &mut $crate::Runtime) -> Result<(), String> {
            $b
            Ok(())
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
