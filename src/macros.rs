
#[macro_export]
macro_rules! dyon_macro_items { ($($x:item)+) => ($($x)+) }

#[macro_export]
macro_rules! dyon_fn {
    (fn $name:ident () -> $rt:ty $b:block) => {
        fn $name(rt: &mut Runtime) -> Result<(), String> {
            rt.push::<$rt>($b);
            Ok(())
        }
    };
    (fn $name:ident ($arg:tt : $t:ty) -> $rt:ty $b:block) => {
        dyon_macro_items!{
            fn $name(rt: &mut Runtime) -> Result<(), String> {
                let $arg: $t = try!(rt.pop());
                rt.push::<$rt>($b);
                Ok(())
            }
        }
    };
    (fn $name:ident ($arg0:tt : $t0:ty, $arg1:tt : $t1:ty) -> $rt:ty $b:block) => {
        dyon_macro_items!{
            fn $name(rt: &mut Runtime) -> Result<(), String> {
                let $arg1: $t1 = try!(rt.pop());
                let $arg0: $t0 = try!(rt.pop());
                rt.push::<$rt>($b);
                Ok(())
            }
        }
    };
    (fn $name:ident () $b:block) => {
        fn $name(_: &mut Runtime) -> Result<(), String> {
            $b
            Ok(())
        }
    };
}
