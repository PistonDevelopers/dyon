use ast;
use Array;
use Variable;
use std::sync::Arc;
use std::collections::HashMap;

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
                    if &**lt == "return" { continue; }
                }
                else {
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
                                return Err(format!("Argument {} does not outlive argument {}",
                                    i, ind));
                            }
                            (Some(a), Some(b)) => {
                                if a <= b {
                                    return Err(format!("Argument {} does not outlive argument {}",
                                        i, ind));
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
    match v {
        &Variable::Ref(ind) => {
            if min.is_none() || min.unwrap() > ind {
                *min = Some(ind);
            }
        }
        &Variable::Return => {}
        &Variable::Bool(_, _) => {}
        &Variable::F64(_, _) => {}
        &Variable::Vec4(_) => {}
        &Variable::Text(_) => {}
        &Variable::Link(_) => {}
        &Variable::UnsafeRef(_) => {}
        &Variable::RustObject(_) => {}
        &Variable::Option(_) => {}
        &Variable::Result(_) => {}
        &Variable::Thread(_) => {}
        &Variable::Array(ref arr) => {
            for v in arr.iter() {
                min_ref(v, min);
            }
        }
        &Variable::Object(ref obj) => {
            for v in obj.values() {
                min_ref(v, min);
            }
        }
        &Variable::Closure(_, _) => {}
        &Variable::InOut(_) => {}
    }
}
