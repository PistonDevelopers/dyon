use range::Range;
use super::node::Node;
use super::kind::Kind;
use Prelude;
use Type;

pub fn run(nodes: &mut Vec<Node>, prelude: &Prelude) -> Result<(), Range<String>> {
    let mut changed;
    loop {
        changed = false;
        for i in 0..nodes.len() {
            if nodes[i].ty.is_some() { continue; }
            let kind = nodes[i].kind;
            let mut this_ty = None;
            match kind {
                Kind::Call => {
                    if let Some(decl) = nodes[i].declaration {
                        if let Some(ref ty) = nodes[decl].ty {
                            this_ty = Some(ty.clone());
                        }
                    }
                    if this_ty.is_none() {
                        if let Some(ref f) = prelude.functions.get(
                                                     nodes[i].name.as_ref().unwrap()) {
                            this_ty = Some(f.ret.clone());
                        }
                    }
                }
                Kind::Mul | Kind::Add | Kind::Return | Kind::Val | Kind::CallArg | Kind::Expr
                | Kind::Cond => {
                    for &ch in &nodes[i].children {
                        if let Some(ref ty) = nodes[ch].ty {
                            this_ty = Some(ty.clone());
                            break;
                        }
                    }
                }
                Kind::Block => {
                    for &ch in nodes[i].children.last() {
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
