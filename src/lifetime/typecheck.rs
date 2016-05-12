use range::Range;
use super::node::Node;
use super::kind::Kind;
use Prelude;
use Type;

pub fn run(nodes: &mut Vec<Node>, prelude: &Prelude) -> Result<(), Range<String>> {
    let mut changed;
    loop {
        changed = false;
        'node: for i in 0..nodes.len() {
            if nodes[i].ty.is_some() { continue; }
            let kind = nodes[i].kind;
            let mut this_ty = None;
            match kind {
                Kind::Call => {
                    if let Some(decl) = nodes[i].declaration {
                        let mut missing = false;
                        for j in 0..nodes[i].children.len() {
                            let ch = nodes[i].children[j];
                            if nodes[ch].item_try_or_ids() { continue 'node; }
                            let arg = nodes[decl].children[j];
                            match (&nodes[ch].ty, &nodes[arg].ty) {
                                (&Some(ref ch_ty), &Some(ref arg_ty)) => {
                                    if !ch_ty.goes_with(arg_ty) {
                                        return Err(nodes[ch].source.wrap(
                                            format!("Type mismatch: Expected `{}`, found `{}`",
                                                arg_ty.description(), ch_ty.description())));
                                    }
                                }
                                (&None, _) | (_, &None) => {
                                    missing = true;
                                }
                            }
                        }
                        if !missing {
                            if let Some(ref ty) = nodes[decl].ty {
                                this_ty = Some(ty.clone());
                            }
                        }
                    }
                    if this_ty.is_none() {
                        if let Some(ref f) = prelude.functions.get(
                                                     nodes[i].name.as_ref().unwrap()) {
                            this_ty = Some(f.ret.clone());
                        }
                    }
                }
                Kind::Assign => {
                    let left = match nodes[i].find_child_by_kind(nodes, Kind::Left) {
                        None => continue,
                        Some(x) => x
                    };
                    let right = match nodes[i].find_child_by_kind(nodes, Kind::Right) {
                        None => continue,
                        Some(x) => x
                    };
                    if nodes[right].item_try_or_ids() { continue 'node; }
                    nodes[left].ty = match (&nodes[left].ty, &nodes[right].ty) {
                        (&None, &Some(ref right_ty)) => {
                            // Make assign return void since there is no more need for checking.
                            this_ty = Some(Type::Void);
                            Some(right_ty.clone())
                        }
                        _ => { continue }
                    };
                    changed = true;
                }
                Kind::Item => {
                    if nodes[i].item_try_or_ids() { continue 'node; }
                    if let Some(decl) = nodes[i].declaration {
                        // No member identifiers.
                        if let Some(ref ty) = nodes[decl].ty {
                            this_ty = Some(ty.clone());
                        }
                    }
                    if let Some(parent) = nodes[i].parent {
                        if nodes[parent].kind == Kind::Left {
                           if let Some(ref ty) = nodes[parent].ty {
                               // Get type from assignment left expression.
                               this_ty = Some(ty.clone());
                           }
                       }
                    }
                }
                Kind::Return | Kind::Val | Kind::CallArg | Kind::Expr
                | Kind::Cond | Kind::Exp | Kind::Base | Kind::Right => {
                    for &ch in &nodes[i].children {
                        if nodes[ch].item_try_or_ids() { continue 'node; }
                        if let Some(ref ty) = nodes[ch].ty {
                            this_ty = Some(ty.clone());
                            break;
                        }
                    }
                }
                Kind::Add => {
                    // Require type to be inferred from all children.
                    let mut it_ty: Option<Type> = None;
                    for &ch in &nodes[i].children {
                        if nodes[ch].item_try_or_ids() { continue 'node; }
                        if let Some(ref ty) = nodes[ch].ty {
                            it_ty = if let Some(ref it) = it_ty {
                                match it.add(ty) {
                                    None => return Err(nodes[ch].source.wrap(
                                        format!("Type mismatch: Binary operator can not be used with `{}` and `{}`",
                                        it.description(), ty.description()))),
                                    x => x
                                }
                            } else {
                                Some(ty.clone())
                            }
                        } else {
                            continue 'node;
                        }
                    }
                    this_ty = it_ty;
                }
                Kind::Mul => {
                    // Require type to be inferred from all children.
                    let mut it_ty: Option<Type> = None;
                    for &ch in &nodes[i].children {
                        if nodes[ch].item_try_or_ids() { continue 'node; }
                        if let Some(ref ty) = nodes[ch].ty {
                            it_ty = if let Some(ref it) = it_ty {
                                match it.mul(ty) {
                                    None => return Err(nodes[ch].source.wrap(
                                        format!("Type mismatch: Binary operator can not be used with `{}` and `{}`",
                                        it.description(), ty.description()))),
                                    x => x
                                }
                            } else {
                                Some(ty.clone())
                            }
                        } else {
                            continue 'node;
                        }
                    }
                    this_ty = it_ty;
                }
                Kind::Pow => {
                    let base = match nodes[i].find_child_by_kind(nodes, Kind::Base) {
                        None => continue 'node,
                        Some(x) => x
                    };
                    let exp = match nodes[i].find_child_by_kind(nodes, Kind::Exp) {
                        None => continue 'node,
                        Some(x) => x
                    };
                    if nodes[base].item_try_or_ids() || nodes[exp].item_try_or_ids() {
                        continue 'node;
                    }
                    match (&nodes[base].ty, &nodes[exp].ty) {
                        (&Some(ref base_ty), &Some(ref exp_ty)) => {
                            if let Some(ty) = base_ty.pow(exp_ty) {
                                this_ty = Some(ty);
                            } else {
                                return Err(nodes[i].source.wrap(
                                    format!("Type mismatch: Binary operator can not be used \
                                             with `{}` and `{}`", base_ty.description(),
                                             exp_ty.description())));
                            }
                        }
                        _ => {}
                    }
                }
                Kind::Block => {
                    for &ch in nodes[i].children.last() {
                        if nodes[ch].item_try_or_ids() { continue 'node; }
                        if let Some(ref ty) = nodes[ch].ty {
                            this_ty = Some(ty.clone());
                            break;
                        }
                    }
                }
                _ => {}
            }
            if this_ty.is_some() {
                nodes[i].ty = this_ty;
                changed = true;
            }
        }
        if !changed { break; }
    }
    for i in 0..nodes.len() {
        let kind = nodes[i].kind;
        match kind {
            Kind::Fn => {
                // TODO: Infer type from body when written as mathematical expression.
                if let Some(ref ty) = nodes[i].ty {
                    try!(check_fn(i, nodes, ty))
                } else {
                    return Err(nodes[i].source.wrap(
                        format!("Could not infer type of function `{}`",
                        nodes[i].name.as_ref().unwrap())
                    ));
                }
            }
            Kind::If => {
                try!(check_if(i, nodes))
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_fn(n: usize, nodes: &Vec<Node>, ty: &Type) -> Result<(), Range<String>> {
    for &ch in &nodes[n].children {
        match nodes[ch].kind {
            Kind::Return => {
                if let Some(ref ret_ty) = nodes[ch].ty {
                    if !ty.goes_with(ret_ty) {
                        return Err(nodes[ch].source.wrap(
                            format!("Type mismatch: Expected `{}`, found `{}`",
                                ty.description(), ret_ty.description())));
                    }
                }
            }
            Kind::ReturnVoid => {
                if !ty.goes_with(&Type::Void) {
                    return Err(nodes[ch].source.wrap(
                        format!("Type mismatch: Expected `{}`, found `{}`",
                            ty.description(), Type::Void.description())));
                }
            }
            _ => {}
        }
        try!(check_fn(ch, nodes, ty));
    }
    Ok(())
}

fn check_if(n: usize, nodes: &Vec<Node>) -> Result<(), Range<String>> {
    for &ch in &nodes[n].children {
        match nodes[ch].kind {
            Kind::Cond => {
                if let Some(ref cond_ty) = nodes[ch].ty {
                    if !Type::Bool.goes_with(cond_ty) {
                        return Err(nodes[ch].source.wrap(
                            format!("Type mismatch: Expected `{}`, found `{}`",
                                Type::Bool.description(), cond_ty.description())));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
