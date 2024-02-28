use super::*;
use crate::Dfn;

use std::sync::Arc;

fn report(
    i: usize,
    found: bool,
    ambiguous: bool,
    count: usize,
    nodes: &[Node],
    todo: &mut Vec<usize>,
) -> Result<(), Range<String>> {
    if !found {
        if ambiguous {
            // Delay completion of this call until extra type information matches.
            todo.push(i);
        } else if count > 0 {
            use std::io::Write;

            let mut buf: Vec<u8> = vec![];
            write!(&mut buf, "Type mismatch (#230):\nThe argument type `").unwrap();
            for (i, arg) in nodes[i]
                .children
                .iter()
                .filter(|&&arg| nodes[arg].kind == Kind::CallArg && !nodes[arg].children.is_empty())
                .map(|&arg| nodes[arg].children[0])
                .enumerate()
            {
                if let Some(ref arg_ty) = nodes[arg].ty {
                    if i != 0 {
                        write!(&mut buf, ", ").unwrap()
                    };
                    write!(&mut buf, "{}", arg_ty.description()).unwrap();
                }
            }
            write!(
                &mut buf,
                "` does not work with `{}`",
                nodes[i].name().expect("Expected name")
            )
            .unwrap();
            return Err(nodes[i].source.wrap(String::from_utf8(buf).unwrap()));
        }
    }
    Ok(())
}

pub(crate) fn declaration(
    i: usize,
    decl: usize,
    nodes: &[Node],
    todo: &mut Vec<usize>,
    this_ty: &mut Option<Type>,
) -> Result<(), Range<String>> {
    // Refine types using extra type information.
    let mut found = false;
    let mut ambiguous = false;
    let mut count = 0;
    'outer: for &ty in nodes[decl]
        .children
        .iter()
        .filter(|&&ty| nodes[ty].kind == Kind::Ty)
    {
        count += 1;
        let mut all = true;
        let mut ty_vars: Vec<Option<Arc<String>>> = vec![None; nodes[ty].names.len()];
        for (arg_expr, &ty_arg) in nodes[i]
            .children
            .iter()
            .filter(|&&arg| nodes[arg].kind == Kind::CallArg && !nodes[arg].children.is_empty())
            .map(|&arg| nodes[arg].children[0])
            .zip(
                nodes[ty]
                    .children
                    .iter()
                    .filter(|&&ty_arg| nodes[ty_arg].kind == Kind::TyArg),
            )
        {
            if nodes[arg_expr].ty.is_none() {
                ambiguous = true;
                break 'outer;
            }
            let found_arg =
                if let (&Some(ref a), &Some(ref b)) = (&nodes[arg_expr].ty, &nodes[ty_arg].ty) {
                    let b = b
                        .bind_ty_vars(a, &nodes[ty].names, &mut ty_vars)
                        .map_err(|err| nodes[arg_expr].source.wrap(err))?;
                    if b.goes_with(a) {
                        if b.ambiguous(a) {
                            ambiguous = true;
                            break 'outer;
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
            if !found_arg {
                all = false;
                break;
            }
        }
        if all {
            if let Some(&ind) = nodes[ty]
                .children
                .iter()
                .find(|&&ty| nodes[ty].kind == Kind::TyRet)
            {
                let mut new_ty = nodes[ind].ty.clone();
                if let Some(ref mut new_ty) = new_ty {
                    for i in 0..nodes[ty].names.len() {
                        if let Some(ref val) = ty_vars[i] {
                            new_ty.insert_var(&nodes[ty].names[i], val);
                        } else {
                            new_ty.insert_none_var(&nodes[ty].names[i]);
                        }
                    }
                }
                *this_ty = new_ty;
                found = true;
                break;
            }
        }
    }

    report(i, found, ambiguous, count, nodes, todo)
}

pub(crate) fn prelude(
    i: usize,
    f: &Dfn,
    nodes: &[Node],
    todo: &mut Vec<usize>,
    this_ty: &mut Option<Type>,
) -> Result<(), Range<String>> {
    // Refine types using extra type information.
    let mut found = false;
    let mut ambiguous = false;
    'outer: for ty in &f.ext {
        let mut all = true;
        let mut ty_vars: Vec<Option<Arc<String>>> = vec![None; ty.0.len()];
        for (arg_expr, ty_arg) in nodes[i]
            .children
            .iter()
            .filter(|&&arg| nodes[arg].kind == Kind::CallArg && !nodes[arg].children.is_empty())
            .map(|&arg| nodes[arg].children[0])
            .zip(ty.1.iter())
        {
            if nodes[arg_expr].ty.is_none() {
                ambiguous = true;
                break 'outer;
            }
            let found_arg = if let Some(ref a) = nodes[arg_expr].ty {
                let ty_arg = ty_arg
                    .bind_ty_vars(a, &ty.0, &mut ty_vars)
                    .map_err(|err| nodes[arg_expr].source.wrap(err))?;
                if ty_arg.goes_with(a) {
                    if ty_arg.ambiguous(a) {
                        ambiguous = true;
                        break 'outer;
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if !found_arg {
                all = false;
                break;
            }
        }
        if all {
            let mut new_ty = ty.2.clone();
            for i in 0..ty.0.len() {
                if let Some(ref val) = ty_vars[i] {
                    new_ty.insert_var(&ty.0[i], val);
                } else {
                    new_ty.insert_none_var(&ty.0[i]);
                }
            }

            *this_ty = Some(new_ty);
            found = true;
            break;
        }
    }

    report(i, found, ambiguous, f.ext.len(), nodes, todo)
}
