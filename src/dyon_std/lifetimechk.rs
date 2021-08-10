use ast;
use std::collections::HashMap;
use std::sync::Arc;
use Array;
use Variable;

/// Performs a runtime lifetime check on arguments.
pub fn check(f: &ast::Function, args: &Array) -> Result<(), String> {
    if f.args.iter().any(|arg| arg.lifetime.is_some()) {
        let mut map: HashMap<Arc<String>, usize> = HashMap::new();
        for (i, arg) in f.args.iter().enumerate() {
            map.insert(arg.name.clone(), i);
        }
        for (i, arg) in f.args.iter().enumerate() {
            if let Some(ref lt) = arg.lifetime {
                if let Variable::Ref(_) = args[i] {
                    if &**lt == "return" {
                        continue;
                    }
                } else {
                    return Err(format!("Expected reference in argument {}", i));
                }
                match map.get(lt) {
                    None => return Err(format!("Something wrong with lifetime `{}`", lt)),
                    Some(&ind) => {
                        let mut left = None;
                        let mut right = None;
                        min_ref(&args[ind], &mut left);
                        min_ref(&args[i], &mut right);
                        match (left, right) {
                            (None, _) => continue,
                            (Some(_), None) => {
                                return Err(format!(
                                    "Argument {} does not outlive argument {}",
                                    i, ind
                                ));
                            }
                            (Some(a), Some(b)) => {
                                if a <= b {
                                    return Err(format!(
                                        "Argument {} does not outlive argument {}",
                                        i, ind
                                    ));
                                } else {
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn min_ref(v: &Variable, min: &mut Option<usize>) {
    use crate::Variable::*;

    match *v {
        Ref(ind) => {
            if min.is_none() || min.unwrap() > ind {
                *min = Some(ind);
            }
        }
        Return => {}
        Bool(_, _) => {}
        F64(_, _) => {}
        Vec4(_) => {}
        Mat4(_) => {}
        Str(_) => {}
        Link(_) => {}
        UnsafeRef(_) => {}
        RustObject(_) => {}
        Option(_) => {}
        Result(_) => {}
        Thread(_) => {}
        Array(ref arr) => {
            for v in arr.iter() {
                min_ref(v, min);
            }
        }
        Object(ref obj) => {
            for v in obj.values() {
                min_ref(v, min);
            }
        }
        Closure(_, _) => {}
        In(_) => {}
    }
}
