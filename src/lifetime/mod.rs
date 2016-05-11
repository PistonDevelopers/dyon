extern crate piston_meta;
extern crate range;

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use self::piston_meta::MetaData;
use self::range::Range;
use self::kind::Kind;
use self::node::{convert_meta_data, Node};
use self::lt::{arg_lifetime, compare_lifetimes, Lifetime};

use prelude::{Lt, Prelude};

mod kind;
mod node;
mod lt;

pub fn check(
    data: &[Range<MetaData>],
    prelude: &Prelude
) -> Result<(), Range<String>> {
    let mut nodes: Vec<Node> = vec![];
    try!(convert_meta_data(&mut nodes, data));

    // Add mutability information to function names.
    for i in 0..nodes.len() {
        match nodes[i].kind {
            Kind::Fn | Kind::Call => {}
            _ => continue
        };
        let mutable_args = nodes[i].children.iter().any(|&arg| nodes[arg].mutable);
        if mutable_args {
            let mut name_plus_args = String::from(&***nodes[i].name.as_ref().unwrap());
            name_plus_args.push('(');
            let mut first = true;
            for &arg in nodes[i].children.iter()
                .filter(|&&n| match nodes[n].kind {
                    Kind::Arg | Kind::CallArg => true, _ => false
                }) {
                if !first { name_plus_args.push(','); }
                name_plus_args.push_str(if nodes[arg].mutable { "mut" } else { "_" });
                first = false;
            }
            name_plus_args.push(')');
            nodes[i].name = Some(Arc::new(name_plus_args));
        }
    }

    // Collect indices to function nodes.
    let functions: Vec<usize> = nodes.iter().enumerate()
        .filter(|&(_, n)| n.kind == Kind::Fn).map(|(i, _)| i).collect();

    // Stores functions arguments with same index as `functions`.
    let mut function_args = Vec::with_capacity(functions.len());

    // Collect indices to call nodes.
    let calls: Vec<usize> = nodes.iter().enumerate()
        .filter(|&(_, n)| n.kind == Kind::Call).map(|(i, _)| i).collect();

    // Collect indices to returns.
    let returns: Vec<usize> = nodes.iter().enumerate()
        .filter(|&(_, n)| n.kind == Kind::Return).map(|(i, _)| i).collect();

    // Collect indices to declared locals.
    // Stores assign node, item node.
    let locals: Vec<(usize, usize)> = nodes.iter().enumerate()
        .filter(|&(_, n)| n.op == Some(Op::Assign)
            && n.children.len() > 0
            && nodes[n.children[0]].children.len() > 0)
        .map(|(i, n)| {
                // Left argument.
                let j = n.children[0];
                let node = &nodes[j];
                // Item in left argument.
                let j = node.children[0];
                (i, j)
            })
        // Filter out assignments to objects or arrays to get locals only.
        .filter(|&(_, j)| nodes[j].ids == 0)
        .collect();

    // Collect indices to mutated locals.
    // Stores assign node, item node.
    let mutated_locals: Vec<(usize, usize)> = nodes.iter().enumerate()
        .filter(|&(_, n)| n.op == Some(Op::Set))
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
    let items: Vec<usize> = nodes.iter().enumerate()
        .filter(|&(i, n)| {
            n.kind == Kind::Item
            && locals.binary_search_by(|&(_, it)| it.cmp(&i)).is_err()
        })
        .map(|(i, _)| i)
        .collect();

    // Link items to their declaration.
    for &i in &items {
        // When `return` is used as variable one does not need to link.
        if nodes[i].name.as_ref().map(|n| &**n == "return") == Some(true) {
            continue;
        }

        // Check with all the parents to find the declaration.
        let mut child = i;
        let mut parent = nodes[i].parent.expect("Expected parent");
        let mut it: Option<usize> = None;

        'search: loop {
            if nodes[parent].kind.is_decl_loop() &&
               nodes[parent].name == nodes[i].name {
               it = Some(parent);
               break 'search;
            }

            let me = nodes[parent].children.binary_search(&child)
                .expect("Expected parent to contain child");
            let children = &nodes[parent].children[..me];
            for &j in children {
                if nodes[j].children.len() == 0 { continue; }
                // Assign is inside an expression.
                let j = nodes[j].children[0];
                if nodes[j].kind != Kind::Assign { continue; }
                let left = nodes[j].children[0];
                let item = nodes[left].children[0];
                if nodes[item].name == nodes[i].name {
                    it = Some(item);
                    break 'search;
                }
            }
            match nodes[parent].parent {
                Some(new_parent) => {
                    child = parent;
                    parent = new_parent;
                }
                None => break
            }
        }

        match it {
            Some(it) => nodes[i].declaration = Some(it),
            None => {
                if nodes[parent].kind != Kind::Fn {
                    panic!("Top parent is not a function");
                }
                if nodes[i].name.is_none() {
                    panic!("Item has no name");
                }

                // Search among function arguments.
                let mut found: Option<usize> = None;
                for &j in &nodes[parent].children {
                    let arg = &nodes[j];
                    if arg.kind != Kind::Arg { continue; }
                    if Some(true) == arg.name.as_ref().map(|n|
                        &**n == &**nodes[i].name.as_ref().unwrap()) {
                        found = Some(j);
                    }
                }
                match found {
                    Some(j) => {
                        nodes[i].declaration = Some(j);
                    }
                    None => {
                        return Err(nodes[i].source.wrap(
                            format!("Could not find declaration of `{}`",
                            nodes[i].name.as_ref().expect("Expected name"))));
                    }
                }
            }
        }
    }

    // Check for duplicate function arguments.
    let mut arg_names: HashSet<Arc<String>> = HashSet::new();
    for &f in &functions {
        arg_names.clear();
        let mut n = 0;
        for &i in nodes[f].children.iter().filter(|&&i| nodes[i].kind == Kind::Arg) {
            let name = nodes[i].name.as_ref().expect("Expected name");
            if arg_names.contains(name) {
                return Err(nodes[i].source.wrap(
                    format!("Duplicate argument `{}`", name)));
            } else {
                arg_names.insert(name.clone());
            }
            n += 1;
        }
        function_args.push(n);
    }

    // Check for duplicate functions and build name to index map.
    let mut function_lookup: HashMap<Arc<String>, usize> = HashMap::new();
    for (i, &f) in functions.iter().enumerate() {
        let name = nodes[f].name.as_ref().expect("Expected name");
        if function_lookup.contains_key(name) {
            return Err(nodes[f].source.wrap(
                format!("Duplicate function `{}`", name)));
        } else {
            function_lookup.insert(name.clone(), i);
        }
    }

    // Link call nodes to functions.
    for &c in &calls {
        let n = {
            nodes[c].children.iter()
            .filter(|&&i| nodes[i].kind == Kind::CallArg)
            .count()
        };

        let node = &mut nodes[c];
        let name = node.name.as_ref().expect("Expected name");
        let i = match function_lookup.get(name) {
            Some(&i) => i,
            None => {
                // Check whether it is a prelude function.
                match prelude.functions.get(name) {
                    Some(pf) => {
                        node.lts = pf.lts.clone();
                        if node.lts.len() != n {
                            return Err(node.source.wrap(
                                format!("{}: Expected {} arguments, found {}",
                                name, node.lts.len(), n)));
                        }
                        continue;
                    }
                    None => {}
                }
                let suggestions = suggestions(&**name, &function_lookup, prelude);
                return Err(node.source.wrap(
                    format!("Could not find function `{}`{}", name, suggestions)));
            }
        };
        // Check that number of arguments is the same as in declaration.
        if function_args[i] != n {
        let suggestions = suggestions(&**name, &function_lookup, prelude);
            return Err(node.source.wrap(
                format!("{}: Expected {} arguments, found {}{}",
                name, function_args[i], n, suggestions)));
        }
        node.declaration = Some(functions[i]);
    }

    // Build a map from (function, argument_name) => (argument, index).
    let mut arg_names: ArgNames = HashMap::new();
    for (i, &f) in functions.iter().enumerate() {
        let function = &nodes[f];
        for (j, &c) in function.children.iter()
            .filter(|&&c| nodes[c].kind == Kind::Arg)
            .enumerate() {
            let name = nodes[c].name.as_ref().expect("Expected name");
            arg_names.insert((f, name.clone()), (c, j));
        }
        // Check that all lifetimes except `'return` points to another argument.
        for &c in function.children.iter()
            .filter(|&&c| nodes[c].kind == Kind::Arg) {
            if let Some(ref lt) = nodes[c].lifetime {
                if &**lt == "return" { continue; }
                if !arg_names.contains_key(&(f, lt.clone())) {
                    return Err(nodes[c].source.wrap(
                        format!("Could not find argument `{}`", lt)));
                }
            }
        }

        // Check for cyclic references among lifetimes.
        let mut visited = vec![false; function_args[i]];
        for (_, &c) in function.children.iter()
            .filter(|&&c| nodes[c].kind == Kind::Arg)
            .enumerate() {
            if let Some(ref lt) = nodes[c].lifetime {
                if &**lt == "return" { break; }
                // Reset visit flags.
                for i in 0..visited.len() { visited[i] = false; }

                let (mut arg, mut ind) = *arg_names.get(&(f, lt.clone()))
                    .expect("Expected argument index");
                loop {
                    if visited[ind] {
                        return Err(nodes[arg].source.wrap(
                                format!("Cyclic lifetime for `{}`", lt)));
                    }
                    visited[ind] = true;

                    // Go to next argument by following the lifetime.
                    let name = match nodes[arg].lifetime {
                            None => break,
                            Some(ref name) => name.clone()
                        };
                    if &**name == "return" { break; }
                    let (new_arg, new_ind) = *arg_names.get(&(f, name))
                        .expect("Expected argument");
                    arg = new_arg;
                    ind = new_ind;
                }
            }
        }
    }

    // Check the lifetime of mutated locals.
    for &(a, i) in &mutated_locals {
        let right = nodes[a].children[1];
        let ref lifetime_left = nodes[i].lifetime(&nodes, &arg_names);
        let ref lifetime_right = nodes[right].lifetime(&nodes, &arg_names);
        try!(compare_lifetimes(lifetime_left, lifetime_right, &nodes)
                .map_err(|err| nodes[right].source.wrap(err)));
    }

    // Check the lifetime of declared locals.
    for &(a, i) in &locals {
        let right = nodes[a].children[1];
        let ref lifetime_left = Some(Lifetime::Local(i));
        let ref lifetime_right = nodes[right].lifetime(&nodes, &arg_names);
        try!(compare_lifetimes(lifetime_left, lifetime_right, &nodes)
                .map_err(|err| nodes[right].source.wrap(err)));
    }

    // Check the lifetime of returned values.
    for &i in &returns {
        let right = nodes[i].children[0];
        let ref lifetime_right = nodes[right].lifetime(&nodes, &arg_names);
        try!(compare_lifetimes(
            &Some(Lifetime::Return(vec![])), lifetime_right, &nodes)
                .map_err(|err| nodes[right].source.wrap(err))
        );
    }

    // Check that calls do not have arguments with shorter lifetime than the call.
    for &c in &calls {
        let call = &nodes[c];
        // Fake a local variable.
        let ref lifetime_left = Some(Lifetime::Local(c));
        for &a in call.children.iter()
            .filter(|&&i| nodes[i].kind == Kind::CallArg)  {
            let ref lifetime_right = nodes[a].lifetime(&nodes, &arg_names);
            try!(compare_lifetimes(lifetime_left, lifetime_right, &nodes)
                    .map_err(|err| nodes[a].source.wrap(err)));
        }
    }

    // Check that calls satisfy the lifetime constraints of arguments.
    for &c in &calls {
        let call = &nodes[c];
        let is_reference = |i: usize| {
            let mut n: usize = call.children[i];
            let mut can_be_item = true;
            // Item is 4 levels down inside arg/add/mul/val
            for _ in 0..4 {
                let node: &Node = &nodes[n];
                if node.kind == Kind::Item { break; }
                if node.children.len() == 0 {
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
            for (i, &a) in function.children.iter()
                .enumerate()
                .filter(|&(_, &i)| nodes[i].kind == Kind::Arg)  {
                let arg = &nodes[a];
                if let Some(ref lt) = arg.lifetime {
                    // When arguments should outlive the return value,
                    // make sure they are referenced.
                    let arg_lifetime = arg_lifetime(a, arg, &nodes, &arg_names);
                    match arg_lifetime {
                        Some(Lifetime::Return(_)) | Some(Lifetime::Argument(_)) => {
                            if !is_reference(i) {
                                return Err(nodes[call.children[i]].source.wrap(
                                    format!("Requires reference to variable")));
                            }
                        }
                        _ => {}
                    }

                    if &**lt != "return" {
                        // Compare the lifetime of the two arguments.
                        let (_, ind) = *arg_names.get(&(declaration, lt.clone()))
                            .expect("Expected argument name");
                        let left = call.children[ind];
                        let right = call.children[i];
                        let ref lifetime_left = nodes[left].lifetime(&nodes, &arg_names);
                        let ref lifetime_right = nodes[right].lifetime(&nodes, &arg_names);
                        try!(compare_lifetimes(
                            lifetime_left, lifetime_right, &nodes)
                                .map_err(|err| nodes[right].source.wrap(err))
                        );
                    }
                }
            }
        } else {
            // Check that call to intrinsic satisfy the declared constraints.
            for ((i, &lt), &call_arg) in
            call.lts.iter().enumerate()
                .zip(call.children.iter()
                .filter(|&&n| nodes[n].kind == Kind::CallArg)) {
                let arg = &nodes[call_arg];
                match lt {
                    Lt::Default => {}
                    Lt::Return => {
                        if !is_reference(i) {
                            return Err(arg.source.wrap(
                                format!("Requires reference to variable")));
                        }
                    }
                    Lt::Arg(ind) => {
                        if !is_reference(i) {
                            return Err(arg.source.wrap(
                                format!("Requires reference to variable")));
                        }

                        let left = call.children[ind];
                        let right = call.children[i];
                        let ref lifetime_left = nodes[left].lifetime(&nodes, &arg_names);
                        let ref lifetime_right = nodes[right].lifetime(&nodes, &arg_names);
                        try!(compare_lifetimes(
                            lifetime_left, lifetime_right, &nodes)
                                .map_err(|err| nodes[right].source.wrap(err))
                        );
                    }
                }
            }
        }
    }

    // Check that mutable locals are not immutable arguments.
    for &(_, i) in &mutated_locals {
        if let Some(decl) = nodes[i].declaration {
            if nodes[decl].kind == Kind::Arg {
                if !nodes[decl].mutable {
                    return Err(nodes[i].source.wrap(
                        format!("Requires `mut {}`", nodes[i].name.as_ref().unwrap())
                    ));
                }
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
                if node.kind == Kind::Item { return Some(n); }
                if node.children.len() == 0 { break; }
                n = node.children[0];
            }
            None
        };

        for &arg in call.children.iter()
            .filter(|&&n| nodes[n].kind == Kind::CallArg
                          && nodes[n].mutable)
        {
            if let Some(n) = reference(arg) {
                if let Some(decl) = nodes[n].declaration {
                   if nodes[decl].kind == Kind::Arg && !nodes[decl].mutable {
                       return Err(nodes[n].source.wrap(
                           format!("Requires `mut {}`", nodes[n].name.as_ref().unwrap())
                       ));
                   }
               }
            }
        }
    }

    Ok(())
}

// Search for suggestions using matching function signature.
// Meant to be put last in error message.
fn suggestions(
    name: &str,
    function_lookup: &HashMap<Arc<String>, usize>,
    prelude: &Prelude
) -> String {
    let search_name = if let Some((mut_pos, _)) = name.chars().enumerate()
        .find(|&(_, c)| c == '(') {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Assign,
    Set,
}
