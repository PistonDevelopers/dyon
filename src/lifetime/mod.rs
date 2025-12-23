extern crate piston_meta;
extern crate range;

use self::kind::Kind;
use self::lt::{arg_lifetime, compare_lifetimes, Lifetime};
use self::node::convert_meta_data;
pub(crate) use self::node::Node;
use self::piston_meta::MetaData;
use self::range::Range;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::ast::{AssignOp, UseLookup};
use crate::prelude::{Lt, Prelude};

use crate::Type;

mod kind;
mod lt;
mod node;
mod normalize;
mod typecheck;

/// Checks lifetime constraints and does type checking.
/// Returns refined return types of functions to put in AST.
pub fn check(
    data: &[Range<MetaData>],
    prelude: &Prelude,
) -> Result<HashMap<Arc<String>, Type>, Range<String>> {
    let mut nodes: Vec<Node> = vec![];
    check_core(&mut nodes, data, prelude)
}

// Core lifetime and type check.
pub(crate) fn check_core(
    nodes: &mut Vec<Node>,
    data: &[Range<MetaData>],
    prelude: &Prelude,
) -> Result<HashMap<Arc<String>, Type>, Range<String>> {
    convert_meta_data(nodes, data)?;

    // Rewrite multiple binary operators into nested ones.
    for i in 0..nodes.len() {
        if nodes[i].binops.len() <= 1 {
            continue;
        };

        let new_child = nodes.len();
        let mut parent = nodes[i].parent;
        for n in (2..nodes[i].children.len()).rev() {
            // The right argument of the last call
            // is the last item among the children.
            // The left argument of the last call
            // is the result of the recursion.
            // The last call gets at the top.
            // This means that it gets pushed first.
            // The left argument points to the next node to be pushed,
            // except the last push which points to original node for reuse.
            let id = nodes.len();
            let right = nodes[i].children[n];
            nodes[right].parent = Some(id);
            nodes.push(Node {
                kind: nodes[i].kind,
                names: vec![],
                ty: None,
                declaration: None,
                alias: None,
                mutable: false,
                try_flag: false,
                grab_level: 0,
                source: nodes[i].source,
                start: nodes[i].start,
                end: nodes[i].end,
                lifetime: None,
                op: None,
                binops: vec![nodes[i].binops[n - 1]],
                lts: vec![],
                parent,
                children: vec![if n == 2 { i } else { id + 1 }, right],
            });
            parent = Some(id);
        }

        // Remove all children from original node except the two first.
        nodes[i].children.truncate(2);
        // Remove all binary operators from original node except the first.
        nodes[i].binops.truncate(1);
        if let Some(old_parent) = nodes[i].parent {
            // Find the node among the children of the parent,
            // but do not rely on binary search because it might be unsorted.
            // Unsorted children is due to possible inference from other rewrites.
            for j in 0..nodes[old_parent].children.len() {
                if nodes[old_parent].children[j] == i {
                    nodes[old_parent].children[j] = new_child;
                    break;
                }
            }
        }
        // Change parent of original node.
        nodes[i].parent = parent;
    }

    // Rewrite graph for syntax sugar that corresponds to function calls.
    for i in 0..nodes.len() {
        if nodes[i].children.len() == 1 {
            match nodes[i].kind {
                Kind::Norm => Node::rewrite_unop(i, crate::NORM.clone(), nodes),
                Kind::Not => Node::rewrite_unop(i, crate::NOT.clone(), nodes),
                Kind::Neg => Node::rewrite_unop(i, crate::NEG.clone(), nodes),
                _ => {}
            }
        } else if nodes[i].binops.len() == 1 && nodes[i].children.len() == 2 {
            use crate::ast::BinOp::*;

            Node::rewrite_binop(
                i,
                match nodes[i].binops[0] {
                    Add => crate::ADD.clone(),
                    Sub => crate::SUB.clone(),
                    Mul => crate::MUL.clone(),
                    Div => crate::DIV.clone(),
                    Rem => crate::REM.clone(),
                    Pow => crate::POW.clone(),
                    Dot => crate::DOT.clone(),
                    Cross => crate::CROSS.clone(),
                    AndAlso => crate::AND_ALSO.clone(),
                    OrElse => crate::OR_ELSE.clone(),
                    Less => crate::LESS.clone(),
                    LessOrEqual => crate::LESS_OR_EQUAL.clone(),
                    Greater => crate::GREATER.clone(),
                    GreaterOrEqual => crate::GREATER_OR_EQUAL.clone(),
                    Equal => crate::EQUAL.clone(),
                    NotEqual => crate::NOT_EQUAL.clone(),
                },
                nodes,
            );
        }

        if nodes[i].children.len() == 1 {
            match nodes[i].kind {
                Kind::Add | Kind::Mul => Node::simplify(i, nodes),
                _ => {}
            }
        }
    }

    // After graph rewrite, the graph might be unnormalized.
    normalize::fix(nodes);

    // Add mutability information to function names.
    for i in 0..nodes.len() {
        match nodes[i].kind {
            Kind::Fn | Kind::Call => {}
            Kind::CallClosure => {
                let word = nodes[i].name().cloned();
                if let Some(ref word) = word {
                    // Append named syntax to item.
                    let item = nodes[i].find_child_by_kind(nodes, Kind::Item).unwrap();
                    if nodes[item].children.is_empty() {
                        Arc::make_mut(&mut nodes[item].names[0]).push_str(&format!("__{}", word));
                    }
                    // Ignore when using object property,
                    // because the key is unknown anyway.
                }
            }
            _ => continue,
        };
        let mutable_args = nodes[i].children.iter().any(|&arg| {
            (nodes[arg].kind == Kind::Arg || nodes[arg].kind == Kind::CallArg) && nodes[arg].mutable
        });
        if mutable_args {
            let mut name_plus_args = String::from(&***nodes[i].name().unwrap());
            name_plus_args.push('(');
            let mut first = true;
            for &arg in nodes[i]
                .children
                .iter()
                .filter(|&&n| matches!(nodes[n].kind, Kind::Arg | Kind::CallArg))
            {
                if !first {
                    name_plus_args.push(',');
                }
                name_plus_args.push_str(if nodes[arg].mutable { "mut" } else { "_" });
                first = false;
            }
            name_plus_args.push(')');
            nodes[i].names = vec![Arc::new(name_plus_args)];
        }
    }

    // Collect indices to function nodes.
    let functions: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|&(_, n)| n.kind == Kind::Fn)
        .map(|(i, _)| i)
        .collect();

    // Stores number of functions arguments with same index as `functions`.
    // To look up number of arguments, use `.enumerate()` on the loop.
    let mut function_args: Vec<usize> = Vec::with_capacity(functions.len());

    // Collect indices to call nodes.
    let calls: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|&(_, n)| n.kind == Kind::Call)
        .map(|(i, _)| i)
        .collect();

    // Collect indices to in-nodes.
    #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
    let ins: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|&(_, n)| n.kind == Kind::In)
        .map(|(i, _)| i)
        .collect();

    // Collect indices to returns.
    let returns: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|&(_, n)| n.kind == Kind::Return)
        .map(|(i, _)| i)
        .collect();

    // Collect indices to expressions in mathematical declared functions.
    let math_expr: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|&(_, n)| {
            if n.kind != Kind::Expr {
                return false;
            }
            if let Some(parent) = n.parent {
                if nodes[parent].kind != Kind::Fn && nodes[parent].kind != Kind::Closure {
                    return false;
                }
            }
            true
        })
        .map(|(i, _)| i)
        .collect();

    // Collect indices to expressions at end of blocks.
    let end_of_blocks: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|&(i, n)| {
            if n.kind == Kind::Expr && n.children.len() == 1 {
                let ch = n.children[0];
                if !nodes[ch].has_lifetime() {
                    return false;
                }
            }
            if let Some(parent) = n.parent {
                if !nodes[parent].kind.is_block() {
                    return false;
                }
                if *nodes[parent].children.last().unwrap() != i {
                    return false;
                }
                true
            } else {
                false
            }
        })
        .map(|(i, _)| i)
        .collect();

    // Collect indices to declared locals.
    // Stores assign node, item node.
    let locals: Vec<(usize, usize)> = nodes
        .iter()
        .enumerate()
        .filter(|&(_, n)| {
            n.op == Some(AssignOp::Assign)
                && !n.children.is_empty()
                && !nodes[n.children[0]].children.is_empty()
        })
        .map(|(i, n)| {
            // Left argument.
            let j = n.children[0];
            let node = &nodes[j];
            // Item in left argument.
            let j = node.children[0];
            (i, j)
        })
        // Filter out assignments to objects or arrays to get locals only.
        .filter(|&(_, j)| !nodes[j].item_ids())
        .collect();

    // Collect indices to assignments to object or arrays.
    let assigned_locals: Vec<(usize, usize)> = nodes
        .iter()
        .enumerate()
        .filter(|&(_, n)| {
            n.op == Some(AssignOp::Assign)
                && !n.children.is_empty()
                && !nodes[n.children[0]].children.is_empty()
        })
        .map(|(i, n)| {
            // Left argument.
            let j = n.children[0];
            let node = &nodes[j];
            // Item in left argument.
            let j = node.children[0];
            (i, j)
        })
        // Filter to get assignments to objects or arrays only.
        .filter(|&(_, j)| nodes[j].item_ids())
        .collect();

    // Collect indices to mutated locals.
    // Stores assign node, item node.
    let mutated_locals: Vec<(usize, usize)> = nodes
        .iter()
        .enumerate()
        .filter(|&(_, n)| n.op.is_some() && n.op != Some(AssignOp::Assign))
        .map(|(i, n)| {
            // Left argument.
            let j = n.children[0];
            let node = &nodes[j];
            // Item in left argument.
            let j = node.children[0];
            (i, j)
        })
        .collect();

    // Collect indices to references that are not declared.
    let items: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|&(i, n)| {
            n.kind == Kind::Item && locals.binary_search_by(|&(_, it)| it.cmp(&i)).is_err()
        })
        .map(|(i, _)| i)
        .collect();

    // Collect indices to inferred ranges.
    let inferred: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|&(_, n)| n.kind.is_decl_loop() && n.find_child_by_kind(nodes, Kind::End).is_none())
        .map(|(i, _)| i)
        .collect();

    // Link items to their declaration.
    for &i in &items {
        // When `return` is used as variable one does not need to link.
        if nodes[i].name().map(|n| &**n == "return") == Some(true) {
            continue;
        }

        // Check with all the parents to find the declaration.
        let mut child = i;
        let mut parent = nodes[i].parent.expect("Expected parent");
        let mut it: Option<usize> = None;
        // The grab level to search for declaration.
        let mut grab = 0;

        'search: loop {
            if nodes[parent].kind.is_decl_loop()
                || nodes[parent].kind.is_decl_un_loop()
                || nodes[parent].kind.is_in_loop()
            {
                let my_name = nodes[i].name().unwrap();
                for name in &nodes[parent].names {
                    if name == my_name {
                        it = Some(parent);
                        break 'search;
                    }
                }
            }

            let me = nodes[parent]
                .children
                .binary_search(&child)
                .expect("Expected parent to contain child");
            let children = &nodes[parent].children[..me];
            for &j in children.iter().rev() {
                if nodes[j].children.is_empty() {
                    continue;
                }
                // Assign is inside an expression.
                let j = nodes[j].children[0];
                if nodes[j].op != Some(AssignOp::Assign) {
                    continue;
                }
                let left = nodes[j].children[0];
                let item = nodes[left].children[0];
                if nodes[item].name() == nodes[i].name() {
                    if nodes[item].item_ids() {
                        continue;
                    }
                    if grab > 0 {
                        return Err(nodes[i].source.wrap(format!(
                            "Grabbed `{}` has same name as variable.\n\
                            Perhaps the grab level is set too high?",
                            nodes[i].name().expect("Expected name")
                        )));
                    }
                    it = Some(item);
                    break 'search;
                }
            }
            match nodes[parent].parent {
                Some(new_parent) => {
                    child = parent;
                    parent = new_parent;
                    if nodes[parent].kind == Kind::Grab {
                        grab = nodes[parent].grab_level;
                        if grab == 0 {
                            grab = 1;
                        }
                    }
                    if nodes[parent].kind == Kind::Closure {
                        if grab == 0 {
                            // Search only in closure environment one level up.
                            break 'search;
                        }
                        for &j in &nodes[parent].children {
                            let arg = &nodes[j];
                            match arg.kind {
                                Kind::Arg | Kind::Current => {}
                                _ => continue,
                            };
                            if Some(true) == arg.name().map(|n| **n == **nodes[i].name().unwrap()) {
                                if grab > 0 {
                                    return Err(nodes[i].source.wrap(format!(
                                        "Grabbed `{}` has same name as closure argument",
                                        nodes[i].name().expect("Expected name")
                                    )));
                                }
                                it = Some(j);
                                break 'search;
                            }
                        }
                        grab -= 1;
                    }
                }
                None => break,
            }
        }

        match it {
            Some(it) => nodes[i].declaration = Some(it),
            None => {
                if nodes[parent].kind != Kind::Fn && nodes[parent].kind != Kind::Closure {
                    panic!("Top parent is not a function");
                }
                if nodes[i].name().is_none() {
                    panic!("Item has no name");
                }

                // Search among function arguments.
                let mut found: Option<usize> = None;
                for &j in &nodes[parent].children {
                    let arg = &nodes[j];
                    match arg.kind {
                        Kind::Arg | Kind::Current => {}
                        _ => continue,
                    };
                    if Some(true) == arg.name().map(|n| **n == **nodes[i].name().unwrap()) {
                        found = Some(j);
                        break;
                    }
                }
                match found {
                    Some(j) => {
                        nodes[i].declaration = Some(j);
                    }
                    None => {
                        return Err(nodes[i].source.wrap(format!(
                            "Could not find declaration of `{}`",
                            nodes[i].name().expect("Expected name")
                        )));
                    }
                }
            }
        }
    }

    // Report ranges that can not be inferred.
    for &inf in &inferred {
        for name in &nodes[inf].names {
            let mut found = false;
            'item: for &i in &items {
                if nodes[i].declaration != Some(inf) {
                    continue 'item;
                }
                if nodes[i].name() != Some(name) {
                    continue 'item;
                }
                let mut ch = i;
                while let Some(parent) = nodes[ch].parent {
                    match nodes[parent].kind {
                        Kind::Id => {
                            found = true;
                            break 'item;
                        }
                        Kind::Val | Kind::Expr => {}
                        Kind::Mul | Kind::Add => {
                            if nodes[parent].children.len() > 1 {
                                continue 'item;
                            }
                        }
                        _ => continue 'item,
                    }
                    ch = parent;
                }
            }

            if !found {
                return Err(nodes[inf]
                    .source
                    .wrap("Can not infer range from body, use `list[i]` syntax".to_string()));
            }
        }
    }

    // Check for duplicate function arguments.
    let mut arg_names: HashSet<Arc<String>> = HashSet::new();
    for &f in &functions {
        arg_names.clear();
        let mut n = 0;
        for &i in nodes[f]
            .children
            .iter()
            .filter(|&&i| nodes[i].kind == Kind::Arg)
        {
            let name = nodes[i].name().expect("Expected name");
            if arg_names.contains(name) {
                return Err(nodes[i]
                    .source
                    .wrap(format!("Duplicate argument `{}`", name)));
            } else {
                arg_names.insert(name.clone());
            }
            n += 1;
        }
        function_args.push(n);
    }

    // Check extra type information.
    for (i, &f) in functions.iter().enumerate() {
        if nodes[f].ty == Some(Type::Void) {
            for &ch in &nodes[f].children {
                if nodes[ch].kind == Kind::Ty {
                    return Err(nodes[ch].source.wrap(format!(
                        "`{}` has extra type information but does not return anything",
                        nodes[f].name().expect("Expected name")
                    )));
                }
            }
        } else if let Some(ref ret_type) = nodes[f].ty {
            let n = function_args[i];
            for &ch in &nodes[f].children {
                if nodes[ch].kind == Kind::Ty {
                    let mut count = 0;
                    let mut arg = 0;
                    for &ty_ch in nodes[ch].children.iter() {
                        if nodes[ty_ch].kind == Kind::TyArg {
                            if arg < n {
                                if let Some(ref ty_arg_ty) = nodes[ty_ch].ty {
                                    while nodes[nodes[f].children[arg]].kind != Kind::Arg {
                                        arg += 1;
                                    }
                                    if arg < n {
                                        if let Some(ref arg_ty) = nodes[nodes[f].children[arg]].ty {
                                            if !arg_ty.goes_with(ty_arg_ty) {
                                                return Err(nodes[ty_ch].source.wrap(format!(
                                                    "The type `{}` does not work with `{}`",
                                                    ty_arg_ty.description(),
                                                    arg_ty.description()
                                                )));
                                            }
                                        }
                                    }
                                }
                                arg += 1;
                            }
                            count += 1;
                        } else if nodes[ty_ch].kind == Kind::TyRet {
                            if let Some(ref ty_ret) = nodes[ty_ch].ty {
                                if !ret_type.goes_with(ty_ret) {
                                    return Err(nodes[ty_ch].source.wrap(format!(
                                        "The type `{}` does not work with `{}`",
                                        ty_ret.description(),
                                        ret_type.description()
                                    )));
                                }
                            }
                        }
                    }
                    if count != n {
                        return Err(nodes[ch].source.wrap(format!(
                            "Expected {} number of arguments, found {}",
                            n, count
                        )));
                    }
                }
            }
        }
    }

    // Check for duplicate functions and build name to index map.
    let mut function_lookup: HashMap<Arc<String>, usize> = HashMap::new();
    for (i, &f) in functions.iter().enumerate() {
        let name = nodes[f].name().expect("Expected name");
        if function_lookup.contains_key(name) {
            return Err(nodes[f]
                .source
                .wrap(format!("Duplicate function `{}`", name)));
        } else {
            function_lookup.insert(name.clone(), i);
        }
    }

    let mut use_lookup: UseLookup = UseLookup::new();
    for node in nodes.iter() {
        if node.kind == Kind::Uses {
            use crate::ast::Uses;
            use piston_meta::bootstrap::Convert;

            let convert = Convert::new(&data[node.start..node.end]);
            if let Ok((_, val)) = Uses::from_meta_data(convert, &mut vec![]) {
                use_lookup = UseLookup::from_uses_prelude(&val, prelude);
            }
            break;
        }
    }

    // Link call nodes to functions.
    for &c in &calls {
        let n = {
            let mut sum = 0;
            for &ch in nodes[c]
                .children
                .iter()
                .filter(|&&i| nodes[i].kind == Kind::CallArg)
            {
                if let Some(sw) = nodes[ch].find_child_by_kind(nodes, Kind::Swizzle) {
                    sum += nodes[sw]
                        .children
                        .iter()
                        .filter(|&&i| {
                            matches!(nodes[i].kind, Kind::Sw0 | Kind::Sw1 | Kind::Sw2 | Kind::Sw3)
                        })
                        .count();
                } else {
                    sum += 1;
                }
            }
            sum
        };

        let node = &mut nodes[c];
        let name = node.name().expect("Expected name").clone();
        if let Some(ref alias) = node.alias {
            use crate::ast::FnAlias;

            // External functions are treated as loaded in prelude.
            if let Some(&FnAlias::Loaded(i)) =
                use_lookup.aliases.get(alias).and_then(|map| map.get(&name))
            {
                node.lts = prelude.list[i].lts.clone();
                continue;
            } else {
                return Err(node
                    .source
                    .wrap(format!("Could not find function `{}::{}`", alias, name)));
            }
        }
        let i = match function_lookup.get(&name) {
            Some(&i) => i,
            None => {
                // Check whether it is a prelude function.
                if let Some(&pf) = prelude.functions.get(&name) {
                    node.lts = prelude.list[pf].lts.clone();
                    if node.lts.len() != n {
                        return Err(node.source.wrap(format!(
                            "{}: Expected {} arguments, found {}",
                            name,
                            node.lts.len(),
                            n
                        )));
                    }
                    continue;
                }
                let suggestions = suggestions(&**name, &function_lookup, prelude);
                return Err(node
                    .source
                    .wrap(format!("Could not find function `{}`{}", name, suggestions)));
            }
        };
        // Check that number of arguments is the same as in declaration.
        if function_args[i] != n {
            let suggestions = suggestions(&**name, &function_lookup, prelude);
            return Err(node.source.wrap(format!(
                "{}: Expected {} arguments, found {}{}",
                name, function_args[i], n, suggestions
            )));
        }
        node.declaration = Some(functions[i]);
    }

    // Check in-nodes.
    #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
    for &c in &ins {
        let node = &mut nodes[c];
        let name = node.name().expect("Expected name").clone();
        if let Some(ref alias) = node.alias {
            use crate::ast::FnAlias;

            // External functions are treated as loaded in prelude.
            if let Some(&FnAlias::Loaded(i)) =
                use_lookup.aliases.get(alias).and_then(|map| map.get(&name))
            {
                node.lts = prelude.list[i].lts.clone();
                continue;
            } else {
                return Err(node
                    .source
                    .wrap(format!("Could not find function `{}::{}`", alias, name)));
            }
        }
        match function_lookup.get(&name) {
            Some(&i) => i,
            None => {
                // Check whether it is a prelude function.
                if prelude.functions.get(&name).is_some() {
                    continue;
                };
                let suggestions = suggestions(&**name, &function_lookup, prelude);
                return Err(node
                    .source
                    .wrap(format!("Could not find function `{}`{}", name, suggestions)));
            }
        };
    }

    // Build a map from (function, argument_name) => (argument, index).
    let mut arg_names: ArgNames = HashMap::new();
    for (i, &f) in functions.iter().enumerate() {
        let function = &nodes[f];
        for (j, &c) in function
            .children
            .iter()
            .filter(|&&c| nodes[c].kind == Kind::Arg)
            .enumerate()
        {
            let name = nodes[c].name().expect("Expected name");
            arg_names.insert((f, name.clone()), (c, j));
        }
        // Check that all lifetimes except `'return` points to another argument.
        for &c in function
            .children
            .iter()
            .filter(|&&c| nodes[c].kind == Kind::Arg)
        {
            if let Some(ref lt) = nodes[c].lifetime {
                if &**lt == "return" {
                    continue;
                }
                if !arg_names.contains_key(&(f, lt.clone())) {
                    return Err(nodes[c]
                        .source
                        .wrap(format!("Could not find argument `{}`", lt)));
                }
            }
        }

        // Check for cyclic references among lifetimes.
        let mut visited = vec![false; function_args[i]];
        for (_, &c) in function
            .children
            .iter()
            .filter(|&&c| nodes[c].kind == Kind::Arg)
            .enumerate()
        {
            if let Some(ref lt) = nodes[c].lifetime {
                if &**lt == "return" {
                    break;
                }
                // Reset visit flags.
                for it in &mut visited {
                    *it = false;
                }

                let (mut arg, mut ind) = *arg_names
                    .get(&(f, lt.clone()))
                    .expect("Expected argument index");
                loop {
                    if visited[ind] {
                        return Err(nodes[arg]
                            .source
                            .wrap(format!("Cyclic lifetime for `{}`", lt)));
                    }
                    visited[ind] = true;

                    // Go to next argument by following the lifetime.
                    let name = match nodes[arg].lifetime {
                        None => break,
                        Some(ref name) => name.clone(),
                    };
                    if &**name == "return" {
                        break;
                    }
                    let (new_arg, new_ind) = *arg_names.get(&(f, name)).expect("Expected argument");
                    arg = new_arg;
                    ind = new_ind;
                }
            }
        }
    }

    // Check the lifetime of mutated locals.
    for &(a, i) in &mutated_locals {
        // Only `=` needs a lifetime check.
        if nodes[a].op != Some(AssignOp::Set) {
            continue;
        }
        let right = nodes[a].children[1];
        let lifetime_left = &nodes[i].lifetime(nodes, &arg_names);
        let lifetime_right = &nodes[right].lifetime(nodes, &arg_names);
        compare_lifetimes(lifetime_left, lifetime_right, nodes)
            .map_err(|err| nodes[right].source.wrap(err))?;
    }

    // Check the lifetime of declared locals.
    for &(a, i) in &locals {
        let right = nodes[a].children[1];
        let lifetime_left = &Ok(Lifetime::Local(i));
        let lifetime_right = &nodes[right].lifetime(nodes, &arg_names);
        compare_lifetimes(lifetime_left, lifetime_right, nodes)
            .map_err(|err| nodes[right].source.wrap(err))?;
    }

    // Check the lifetime of assigned locals.
    for &(a, i) in &assigned_locals {
        if let Some(j) = nodes[i].declaration {
            let right = nodes[a].children[1];
            let lifetime_left = &Ok(Lifetime::Local(j));
            let lifetime_right = &nodes[right].lifetime(nodes, &arg_names);
            compare_lifetimes(lifetime_left, lifetime_right, nodes)
                .map_err(|err| nodes[right].source.wrap(err))?;
        }
    }

    // Check the lifetime of returned values.
    for &i in &returns {
        let right = nodes[i].children[0];
        let lifetime_right = &nodes[right].lifetime(nodes, &arg_names);
        compare_lifetimes(&Ok(Lifetime::Return(vec![])), lifetime_right, nodes)
            .map_err(|err| nodes[right].source.wrap(err))?;
    }

    // Check the lifetime of expressions that are mathematically declared.
    for &i in &math_expr {
        let lifetime_right = &nodes[i].lifetime(nodes, &arg_names);
        compare_lifetimes(&Ok(Lifetime::Return(vec![])), lifetime_right, nodes)
            .map_err(|err| nodes[i].source.wrap(err))?;
    }

    // Check that no function argument has lifetime "'return"
    // while the function does not return anything.
    for &f in &functions {
        if let Some(Type::Void) = nodes[f].ty {
            for &j in nodes[f]
                .children
                .iter()
                .filter(|&&i| nodes[i].kind == Kind::Arg)
            {
                if let Some(ref lt) = nodes[j].lifetime {
                    if &**lt == "return" {
                        let name = nodes[j].name().expect("Expected name");
                        return Err(nodes[j].source.wrap(format!(
                            "`{}: 'return` , but function does not return",
                            name
                        )));
                    }
                }
            }
        }
    }

    // Check the lifetime of expressions at end of blocks.
    for &i in &end_of_blocks {
        let parent = nodes[i].parent.unwrap();
        // Fake a local variable.
        let lifetime_left = &Ok(Lifetime::Local(parent));
        let lifetime_right = &nodes[i].lifetime(nodes, &arg_names);
        compare_lifetimes(lifetime_left, lifetime_right, nodes)
            .map_err(|err| nodes[i].source.wrap(err))?;
    }

    // Check that calls do not have arguments with shorter lifetime than the call.
    for &c in &calls {
        let call = &nodes[c];
        // Fake a local variable.
        let lifetime_left = &Ok(Lifetime::Local(c));
        for &a in call
            .children
            .iter()
            .filter(|&&i| nodes[i].kind == Kind::CallArg)
        {
            let lifetime_right = &nodes[a].lifetime(nodes, &arg_names);
            compare_lifetimes(lifetime_left, lifetime_right, nodes)
                .map_err(|err| nodes[a].source.wrap(err))?;
        }
    }

    // Check that `go` functions does not have lifetime constraints.
    for &c in &calls {
        let call = &nodes[c];
        #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
        if let Some(parent) = call.parent {
            if nodes[parent].kind != Kind::Go {
                continue;
            }
        } else {
            continue;
        }
        #[cfg(any(target_family = "wasm", not(feature = "threading")))]
        if call.parent.is_none() {
            continue;
        }
        if let Some(declaration) = call.declaration {
            let function = &nodes[declaration];
            for (i, &a) in function
                .children
                .iter()
                .enumerate()
                .filter(|&(_, &i)| nodes[i].kind == Kind::Arg)
            {
                let arg = &nodes[a];
                if arg.lifetime.is_some() {
                    return Err(nodes[call.children[i]].source.wrap(
                        "Can not use `go` because this argument has a lifetime constraint"
                            .to_string(),
                    ));
                }
            }
        } else {
            // Check that call to intrinsic satisfy the declared constraints.
            for ((i, &lt), _) in call.lts.iter().enumerate().zip(
                call.children
                    .iter()
                    .filter(|&&n| nodes[n].kind == Kind::CallArg),
            ) {
                match lt {
                    Lt::Default => {}
                    _ => {
                        return Err(nodes[call.children[i]].source.wrap(
                            "Can not use `go` because this argument has a lifetime constraint"
                                .to_string(),
                        ));
                    }
                }
            }
        }
    }

    // Check that calls satisfy the lifetime constraints of arguments.
    for &c in &calls {
        let call = &nodes[c];
        let map_arg_call_arg_index = |i: usize| {
            // Map `i` to the call arg taking swizzling into account.
            let mut j = 0;
            let mut new_i = 0;
            for &ch in &call.children {
                if let Some(sw) = nodes[ch].find_child_by_kind(nodes, Kind::Swizzle) {
                    for &ch in &nodes[sw].children {
                        j += match nodes[ch].kind {
                            Kind::Sw0 | Kind::Sw1 | Kind::Sw2 | Kind::Sw3 => 1,
                            _ => 0,
                        }
                    }
                } else {
                    j += 1;
                }
                if j > i {
                    break;
                }
                new_i += 1;
            }
            new_i
        };
        let is_reference = |i: usize| {
            let mut n: usize = call.children[i];
            let mut can_be_item = true;
            // Item is some levels down inside arg/add/expr/mul/val
            loop {
                let node: &Node = &nodes[n];
                match node.kind {
                    Kind::Item => break,
                    Kind::Call => {
                        can_be_item = false;
                        break;
                    }
                    _ => {}
                }
                if node.children.is_empty() {
                    can_be_item = false;
                    break;
                }
                n = node.children[0];
            }
            if can_be_item && nodes[n].kind != Kind::Item {
                can_be_item = false;
            }
            can_be_item
        };

        if let Some(declaration) = call.declaration {
            let function = &nodes[declaration];
            for (i, &a) in function
                .children
                .iter()
                .enumerate()
                .filter(|&(_, &i)| nodes[i].kind == Kind::Arg)
                .map(|(i, a)| (map_arg_call_arg_index(i), a))
            {
                let arg = &nodes[a];
                if let Some(ref lt) = arg.lifetime {
                    // When arguments should outlive the return value,
                    // make sure they are referenced.
                    let arg_lifetime = arg_lifetime(a, arg, nodes, &arg_names);
                    match arg_lifetime {
                        Ok(Lifetime::Return(_)) | Ok(Lifetime::Argument(_)) => {
                            if !is_reference(i) {
                                return Err(nodes[call.children[i]]
                                    .source
                                    .wrap("Requires reference to variable".to_string()));
                            }
                        }
                        _ => {}
                    }

                    if &**lt != "return" {
                        // Compare the lifetime of the two arguments.
                        let (_, ind) = *arg_names
                            .get(&(declaration, lt.clone()))
                            .expect("Expected argument name");
                        let left = call.children[ind];
                        let right = call.children[i];
                        let lifetime_left = &nodes[left].lifetime(nodes, &arg_names);
                        let lifetime_right = &nodes[right].lifetime(nodes, &arg_names);
                        compare_lifetimes(lifetime_left, lifetime_right, nodes)
                            .map_err(|err| nodes[right].source.wrap(err))?;
                    }
                }
            }
        } else {
            // Check that call to intrinsic satisfy the declared constraints.
            for (i, &lt) in call
                .lts
                .iter()
                .enumerate()
                .map(|(i, a)| (map_arg_call_arg_index(i), a))
            {
                let arg = &nodes[call.children[i]];
                match lt {
                    Lt::Default => {}
                    Lt::Return => {
                        if !is_reference(i) {
                            return Err(arg
                                .source
                                .wrap("Requires reference to variable".to_string()));
                        }
                    }
                    Lt::Arg(ind) => {
                        if !is_reference(i) {
                            return Err(arg
                                .source
                                .wrap("Requires reference to variable".to_string()));
                        }

                        let left = call.children[ind];
                        let right = call.children[i];
                        let lifetime_left = &nodes[left].lifetime(nodes, &arg_names);
                        let lifetime_right = &nodes[right].lifetime(nodes, &arg_names);
                        compare_lifetimes(lifetime_left, lifetime_right, nodes)
                            .map_err(|err| nodes[right].source.wrap(err))?;
                    }
                }
            }
        }
    }

    // Check that mutable locals are not immutable arguments.
    for &(_, i) in &mutated_locals {
        if let Some(decl) = nodes[i].declaration {
            if (nodes[decl].kind == Kind::Arg || nodes[decl].kind == Kind::Current)
                && !nodes[decl].mutable
            {
                return Err(nodes[i]
                    .source
                    .wrap(format!("Requires `mut {}`", nodes[i].name().unwrap())));
            }
        }
    }

    // Check that calling mutable argument are not immutable.
    for &c in &calls {
        let call = &nodes[c];
        let reference = |i: usize| {
            let mut n: usize = i;
            // Item is 2 levels down inside call_arg/item
            for _ in 0..2 {
                let node: &Node = &nodes[n];
                if node.kind == Kind::Item {
                    return Some(n);
                }
                if node.children.is_empty() {
                    break;
                }
                n = node.children[0];
            }
            None
        };

        for &arg in call
            .children
            .iter()
            .filter(|&&n| nodes[n].kind == Kind::CallArg && nodes[n].mutable)
        {
            if let Some(n) = reference(arg) {
                if let Some(decl) = nodes[n].declaration {
                    if (nodes[decl].kind == Kind::Arg || nodes[decl].kind == Kind::Current)
                        && !nodes[decl].mutable
                    {
                        return Err(nodes[n]
                            .source
                            .wrap(format!("Requires `mut {}`", nodes[n].name().unwrap())));
                    }
                }
            }
        }
    }

    typecheck::run(nodes, prelude, &use_lookup)?;

    // Copy refined return types to use in AST.
    let mut refined_rets: HashMap<Arc<String>, Type> = HashMap::new();
    for (name, &ind) in &function_lookup {
        if let Some(ref ty) = nodes[functions[ind]].ty {
            refined_rets.insert(name.clone(), ty.clone());
        }
    }

    Ok(refined_rets)
}

// Search for suggestions using matching function signature.
// Meant to be put last in error message.
fn suggestions(
    name: &str,
    function_lookup: &HashMap<Arc<String>, usize>,
    prelude: &Prelude,
) -> String {
    let search_name = if let Some((mut_pos, _)) = name.chars().enumerate().find(|&(_, c)| c == '(')
    {
        &name[..mut_pos - 1]
    } else {
        name
    };
    let mut found_suggestions = false;
    let mut suggestions = String::from("\n\nDid you mean:\n");
    for f in function_lookup.keys() {
        if (&***f).starts_with(search_name) {
            suggestions.push_str("- ");
            suggestions.push_str(f);
            suggestions.push('\n');
            found_suggestions = true;
        }
    }
    for f in prelude.functions.keys() {
        if (&***f).starts_with(search_name) {
            suggestions.push_str("- ");
            suggestions.push_str(f);
            suggestions.push('\n');
            found_suggestions = true;
        }
    }
    if found_suggestions {
        suggestions
    } else {
        String::from("")
    }
}

/// Maps (function, argument_name) => (argument, index)
pub type ArgNames = HashMap<(usize, Arc<String>), (usize, usize)>;

pub(crate) fn to_array(nodes: &[Node]) -> Vec<crate::Variable> {
    use crate::embed::PushVariable;
    use crate::Variable;

    let mut res = vec![];
    lazy_static! {
        static ref KIND: Arc<String> = Arc::new("kind".into());
        static ref CHILDREN: Arc<String> = Arc::new("children".into());
        static ref NAMES: Arc<String> = Arc::new("names".into());
        static ref PARENT: Arc<String> = Arc::new("parent".into());
        static ref TY: Arc<String> = Arc::new("ty".into());
        static ref ALIAS: Arc<String> = Arc::new("alias".into());
        static ref MUTABLE: Arc<String> = Arc::new("mutable".into());
        static ref TRY: Arc<String> = Arc::new("try".into());
        static ref GRAB_LEVEL: Arc<String> = Arc::new("grab_level".into());
        static ref SOURCE_OFFSET: Arc<String> = Arc::new("source_offset".into());
        static ref SOURCE_LENGTH: Arc<String> = Arc::new("source_length".into());
        static ref START: Arc<String> = Arc::new("start".into());
        static ref END: Arc<String> = Arc::new("end".into());
        static ref LIFETIME: Arc<String> = Arc::new("lifetime".into());
        static ref DECLARATION: Arc<String> = Arc::new("declaration".into());
        static ref OP: Arc<String> = Arc::new("op".into());
        static ref LTS: Arc<String> = Arc::new("lts".into());
    }
    for n in nodes {
        let mut obj = HashMap::new();
        obj.insert(KIND.clone(), format!("{:?}", n.kind).push_var());
        obj.insert(CHILDREN.clone(), n.children.push_var());
        obj.insert(NAMES.clone(), n.names.push_var());
        obj.insert(PARENT.clone(), n.parent.push_var());
        obj.insert(
            TY.clone(),
            n.ty.as_ref().map(|ty| ty.description()).push_var(),
        );
        obj.insert(ALIAS.clone(), n.alias.push_var());
        obj.insert(MUTABLE.clone(), n.mutable.push_var());
        obj.insert(TRY.clone(), n.try_flag.push_var());
        obj.insert(GRAB_LEVEL.clone(), (n.grab_level as u32).push_var());
        obj.insert(SOURCE_OFFSET.clone(), n.source.offset.push_var());
        obj.insert(SOURCE_LENGTH.clone(), n.source.length.push_var());
        obj.insert(START.clone(), n.start.push_var());
        obj.insert(END.clone(), n.end.push_var());
        obj.insert(LIFETIME.clone(), n.lifetime.push_var());
        obj.insert(DECLARATION.clone(), n.declaration.push_var());
        obj.insert(
            OP.clone(),
            n.op.as_ref().map(|op| format!("{:?}", op)).push_var(),
        );
        obj.insert(
            LTS.clone(),
            n.lts
                .iter()
                .map(|lt| format!("{:?}", lt))
                .collect::<Vec<String>>()
                .push_var(),
        );
        res.push(Variable::Object(Arc::new(obj)));
    }
    res
}
