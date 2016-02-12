extern crate piston_meta;
extern crate range;

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::cmp::{PartialOrd, Ordering};
use self::piston_meta::MetaData;
use self::range::Range;

pub fn check(data: &[Range<MetaData>]) -> Result<(), String> {
    let mut nodes: Vec<Node> = vec![];
    let mut parents: Vec<usize> = vec![];
    for (i, d) in data.iter().enumerate() {
        match d.data {
            MetaData::StartNode(ref kind) => {
                let kind = match Kind::new(kind) {
                    Some(kind) => kind,
                    None => panic!("Unknown kind `{}`", kind)
                };

                let parent = parents.last().map(|i| *i);
                parents.push(nodes.len());
                nodes.push(Node {
                    kind: kind,
                    name: None,
                    source: Range::empty(0),
                    parent: parent,
                    children: vec![],
                    start: i,
                    end: 0,
                    lifetime: None,
                    declaration: None,
                    op: None,
                    ids: 0,
                    arg_constraints: vec![]
                });
            }
            MetaData::EndNode(_) => {
                let ind = parents.pop().unwrap();
                {
                    let node = &mut nodes[ind];
                    node.source = d.range();
                    node.end = i + 1;
                }
                match parents.last() {
                    Some(&parent) => {
                        nodes[parent].children.push(ind);
                    }
                    None => {}
                }
            }
            MetaData::String(ref n, ref val) => {
                match &***n {
                    "name" => {
                        let i = *parents.last().unwrap();
                        nodes[i].name = Some(val.clone());
                    }
                    "word" => {
                        // Put words together to name.
                        let i = *parents.last().unwrap();
                        let ref mut name = nodes[i].name;
                        if let &mut Some(ref mut name) = name {
                            let name = Arc::make_mut(name);
                            name.push('_');
                            name.push_str(val);
                        } else {
                            *name = Some(val.clone());
                        }
                    }
                    "lifetime" => {
                        let i = *parents.last().unwrap();
                        nodes[i].lifetime = Some(val.clone());
                    }
                    "id" => {
                        let i = *parents.last().unwrap();
                        nodes[i].ids += 1;
                    }
                    _ => {}
                }
            }
            MetaData::Bool(ref n, _val) => {
                match &***n {
                    ":=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].op = Some(Op::Assign);
                    }
                    "=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].op = Some(Op::Set);
                    }
                    _ => {}
                }
            }
            _ => {}
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
        .filter(|&(_, n)| n.op == Some(Op::Assign))
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
                        return Err(format!("Could not find declaration of `{}`",
                            nodes[i].name.as_ref().expect("Expected name")));
                    }
                }
            }
        }
    }

    // List all intrinsic functions.
    // Stores name, number of arguments, returns
    let mut intrinsics: HashMap<&'static str, Intrinsic> = HashMap::new();
    intrinsics.insert("println", PRINTLN);
    intrinsics.insert("print", PRINT);
    intrinsics.insert("clone", CLONE);
    intrinsics.insert("debug", DEBUG);
    intrinsics.insert("backtrace", BACKTRACE);
    intrinsics.insert("sleep", SLEEP);
    intrinsics.insert("round", ROUND);
    intrinsics.insert("random", RANDOM);
    intrinsics.insert("read_number", READ_NUMBER);
    intrinsics.insert("read_line", READ_LINE);
    intrinsics.insert("len", LEN);
    intrinsics.insert("push", PUSH);
    intrinsics.insert("trim_right", TRIM_RIGHT);
    intrinsics.insert("to_string", TO_STRING);
    intrinsics.insert("sqrt", SQRT);
    intrinsics.insert("sin", SIN);
    intrinsics.insert("asin", ASIN);
    intrinsics.insert("cos", COS);
    intrinsics.insert("acos", ACOS);
    intrinsics.insert("tan", TAN);
    intrinsics.insert("atan", ATAN);
    intrinsics.insert("exp", EXP);
    intrinsics.insert("ln", LN);
    intrinsics.insert("log2", LOG2);
    intrinsics.insert("log10", LOG10);
    intrinsics.insert("random", RANDOM);

    // Check for duplicate function arguments.
    let mut arg_names: HashSet<Arc<String>> = HashSet::new();
    for &f in &functions {
        arg_names.clear();
        let mut n = 0;
        for &i in nodes[f].children.iter().filter(|&&i| nodes[i].kind == Kind::Arg) {
            let name = nodes[i].name.as_ref().expect("Expected name");
            if arg_names.contains(name) {
                return Err(format!("Duplicate argument `{}`", name));
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
            return Err(format!("Duplicate function `{}`", name));
        } else {
            function_lookup.insert(name.clone(), i);
        }
    }

    // Link call nodes to functions.
    for &c in &calls {
        let n = {
            nodes[c].children.iter()
            .filter(|&&i| nodes[i].kind == Kind::Arg)
            .count()
        };

        let node = &mut nodes[c];
        let name = node.name.as_ref().expect("Expected name");
        let i = match function_lookup.get(name) {
            Some(&i) => i,
            None => {
                // Check whether it is an intrinsic operation.
                match intrinsics.get(&***name) {
                    Some(intr) => {
                        // Copy argument constraints to use when computing
                        // lifetimes.
                        node.arg_constraints = intr.arg_constraints.into();
                        continue;
                    }
                    None => {
                        return Err(format!("Could not find function `{}`", name));
                    }
                }
            }
        };
        // Check that number of arguments is the same as in declaration.
        if function_args[i] != n {
            return Err(format!("{}: Expected {} arguments, found {}",
                name, function_args[i], n));
        }
        node.declaration = Some(i);
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
                    return Err(format!("Could not find argument `{}`", lt));
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
                        return Err(format!("Cyclic lifetime for `{}`", lt));
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
        let lifetime_left = nodes[i].lifetime(&nodes, &arg_names);
        let lifetime_right = nodes[right].lifetime(&nodes, &arg_names);
        try!(compare_lifetimes(lifetime_left, lifetime_right, &nodes));
    }

    // Check the lifetime of returned values.
    for &i in &returns {
        let right = nodes[i].children[0];
        let lifetime_right = nodes[right].lifetime(&nodes, &arg_names);
        try!(compare_lifetimes(Some(Lifetime::Return(vec![])), lifetime_right, &nodes));
    }

    // Check that calls satisfy the lifetime constraints of arguments.
    for &c in &calls {
        let call = &nodes[c];
        if let Some(declaration) = call.declaration {
            let function = &nodes[declaration];
            for (i, &a) in function.children.iter()
                .filter(|&&i| nodes[i].kind == Kind::Arg)
                .enumerate() {
                let arg = &nodes[a];
                if let Some(ref lt) = arg.lifetime {
                    // When arguments should outlive the return value,
                    // make sure they are referenced.
                    let arg_lifetime = arg_lifetime(a, arg, &nodes, &arg_names);
                    if let Some(Lifetime::Return(_)) = arg_lifetime {
                        let mut n = call.children[i];
                        let mut can_be_item = true;
                        // Item is 4 levels down inside arg/add/mul/val
                        for _ in 0..4 {
                            let node = &nodes[n];
                            if node.children.len() == 0 {
                                can_be_item = false;
                                break;
                            }
                            n = nodes[n].children[0];
                        }
                        if can_be_item && nodes[n].kind != Kind::Item {
                            can_be_item = false;
                        }
                        if !can_be_item {
                            return Err(format!("Requires reference to variable"));
                        }
                    }

                    if &**lt != "return" {
                        // Compare the lifetime of the two arguments.
                        let (_, ind) = *arg_names.get(&(declaration, lt.clone()))
                            .expect("Expected argument name");
                        let left = call.children[ind];
                        let right = call.children[i];
                        let lifetime_left = nodes[left].lifetime(&nodes, &arg_names);
                        let lifetime_right = nodes[right].lifetime(&nodes, &arg_names);
                        try!(compare_lifetimes(lifetime_left, lifetime_right, &nodes));
                    }
                }
            }
        }
    }

    Ok(())
}

fn compare_lifetimes(
    l: Option<Lifetime>,
    r: Option<Lifetime>,
    nodes: &Vec<Node>
) -> Result<(), String> {
    match (l, r) {
        (Some(l), Some(r)) => {
            match l.partial_cmp(&r) {
                Some(Ordering::Greater) => {
                    match r {
                        Lifetime::Local(r) => {
                            return Err(format!("`{}` does not live long enough",
                                nodes[r].name.as_ref().expect("Expected name")));
                        }
                        Lifetime::Argument(ref r) => {
                            return Err(format!("`{}` does not live long enough",
                                nodes[r[0]].name.as_ref().expect("Expected name")));
                        }
                        _ => unimplemented!()
                    }
                }
                None => {
                    match (l, r) {
                        (Lifetime::Argument(ref l), Lifetime::Argument(ref r)) => {
                            return Err(format!("Requires `{}: '{}`",
                                nodes[r[0]].name.as_ref().expect("Expected name"),
                                nodes[l[0]].name.as_ref().expect("Expected name")));
                        }
                        (Lifetime::Argument(ref l), Lifetime::Return(ref r)) => {
                            if r.len() > 0 {
                                return Err(format!("Requires `{}: '{}`",
                                    nodes[r[0]].name.as_ref().expect("Expected name"),
                                    nodes[l[0]].name.as_ref().expect("Expected name")));
                            } else {
                                unimplemented!();
                            }
                        }
                        (Lifetime::Return(ref l), Lifetime::Return(ref r)) => {
                            if l.len() > 0 && r.len() > 0 {
                                return Err(format!("Requires `{}: '{}`",
                                    nodes[r[0]].name.as_ref().expect("Expected name"),
                                    nodes[l[0]].name.as_ref().expect("Expected name")));
                            } else {
                                unimplemented!();
                            }
                        }
                        (Lifetime::Return(ref l), Lifetime::Argument(ref r)) => {
                            if l.len() == 0 {
                                let last = *r.last().expect("Expected argument index");
                                return Err(format!("Requires `{}: 'return`",
                                    nodes[last].name.as_ref().expect("Expected name")));
                            } else {
                                unimplemented!();
                            }
                        }
                        x => panic!("Unknown case {:?}", x)
                    }
                }
                _ => {}
            }
        }
        // TODO: Handle other cases.
        _ => {}
    }
    Ok(())
}

/// Maps (function, argument_name) => (argument, index)
pub type ArgNames = HashMap<(usize, Arc<String>), (usize, usize)>;

#[derive(Debug)]
pub struct Node {
    /// The kind of node.
    pub kind: Kind,
    /// The name.
    pub name: Option<Arc<String>>,
    /// The range in source.
    pub source: Range,
    /// The parent index.
    pub parent: Option<usize>,
    /// The children.
    pub children: Vec<usize>,
    /// The start index in meta data.
    pub start: usize,
    /// The end index in meta data.
    pub end: usize,
    /// The lifetime.
    pub lifetime: Option<Arc<String>>,
    /// The declaration.
    pub declaration: Option<usize>,
    /// Operation.
    pub op: Option<Op>,
    /// Number of ids.
    /// Used to determine declaration of locals.
    pub ids: u32,
    /// The argument constraints, one for each argument to a function.
    /// Just using an empty vector for nodes that are not functions.
    pub arg_constraints: Vec<ArgConstraint>,
}

fn arg_lifetime(
    declaration: usize,
    arg: &Node,
    nodes: &[Node],
    arg_names: &ArgNames
) -> Option<Lifetime> {
    return Some(if let Some(ref lt) = arg.lifetime {
        if &**lt == "return" {
            return Some(Lifetime::Return(vec![declaration]));
        } else {
            // Resolve lifetimes among arguments.
            let parent = arg.parent.expect("Expected parent");
            let mut args: Vec<usize> = vec![];
            args.push(declaration);
            let mut name = lt.clone();
            loop {
                let (arg, _) = *arg_names.get(&(parent, name))
                    .expect("Expected argument name");
                args.push(arg);
                if let Some(ref lt) = nodes[arg].lifetime {
                    if &**lt == "return" {
                        // Lifetimes outlive return.
                        return Some(Lifetime::Return(args));
                    }
                    name = lt.clone();
                } else {
                    break;
                }
            }
            Lifetime::Argument(args)
        }
    } else {
        Lifetime::Argument(vec![declaration])
    })
}

impl Node {
    pub fn print(&self, nodes: &[Node], indent: u32) {
        for _ in 0..indent { print!(" ") }
        println!("{:?} {:?} {{", self.kind, self.name);
        for &c in &self.children {
            nodes[c].print(nodes, indent + 1);
        }
        for _ in 0..indent { print!(" ") }
        println!("}}")
    }

    pub fn lifetime(
        &self,
        nodes: &[Node],
        arg_names: &ArgNames
    ) -> Option<Lifetime> {
        if let Some(declaration) = self.declaration {
            if self.kind == Kind::Item {
                let arg = &nodes[declaration];
                if arg.kind == Kind::Arg {
                    return arg_lifetime(declaration, &arg, nodes, arg_names);
                } else {
                    return Some(Lifetime::Local(declaration));
                }
            }
        } else {
            // Intrinsic functions copies argument constraints to the call.
            if self.kind == Kind::Call && self.arg_constraints.len() > 0 {
                let mut returns_static = true;
                'args: for (_, arg) in self.children.iter().map(|&i| &nodes[i])
                        .filter(|&n| n.kind == Kind::Arg)
                        .zip(self.arg_constraints.iter()) {
                    let mut arg = *arg;
                    loop {
                        match arg {
                            ArgConstraint::Default => {
                                continue 'args;
                            }
                            ArgConstraint::Return => {
                                returns_static = false;
                                break 'args;
                            }
                            x => {
                                arg = x;
                                continue;
                            }
                        }
                    }
                }

                if returns_static {
                    return None;
                }
            } else if self.kind == Kind::Item
                && self.name.as_ref().map(|n| &**n == "return") == Some(true) {
                return Some(Lifetime::Return(vec![]));
            }
        }

        let mut min: Option<Lifetime> = None;
        // TODO: Filter by kind of children.
        for &c in &self.children {
            match (self.kind, nodes[c].kind) {
                (_, Kind::Object) => {}
                (_, Kind::KeyValue) => {}
                (_, Kind::Val) => {}
                (_, Kind::Add) => {}
                (_, Kind::Mul) => {}
                (_, Kind::Call) => {}
                (_, Kind::Item) => {}
                (_, Kind::UnOp) => {}
                (_, Kind::Expr) => {}
                (_, Kind::Array) => {}
                (_, Kind::ArrayItem) => {}
                x => panic!("Unimplemented `{:?}`", x),
            }
            let lifetime = match nodes[c].lifetime(nodes, arg_names) {
                Some(lifetime) => lifetime,
                None => { continue; }
            };
            if min.is_none() || min.as_ref().map(|l| l < &lifetime) == Some(true) {
                min = Some(lifetime);
            }
        }
        min
    }
}

/// Describes the lifetime of a variable.
/// When a lifetime `a` > `b` it means `a` outlives `b`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lifetime {
    /// Return value with optional list of arguments that outlives other arguments.
    Return(Vec<usize>),
    /// An argument outlives other arguments, but does not outlive the return.
    Argument(Vec<usize>),
    /// Local variable.
    Local(usize),
}

impl PartialOrd for Lifetime {
    fn partial_cmp(&self, other: &Lifetime) -> Option<Ordering> {
        use self::Lifetime::*;

        Some(match (self, other) {
            (&Local(a), &Local(b)) => b.cmp(&a),
            (&Return(_), &Local(_)) => Ordering::Greater,
            (&Local(_), &Return(_)) => Ordering::Less,
            (&Return(ref a), &Return(ref b)) => {
                match (a.len(), b.len()) {
                    (0, 0) => Ordering::Equal,
                    (0, _) => Ordering::Less,
                    (_, 0) => Ordering::Greater,
                    (_, _) => {
                        return compare_argument_outlives(a, b);
                    }
                }
            }
            (&Argument(_), &Local(_)) => Ordering::Greater,
            (&Local(_), &Argument(_)) => Ordering::Less,
            (&Return(_), &Argument(_)) => return None,
            (&Argument(_), &Return(_)) => return None,
            (&Argument(ref a), &Argument(ref b)) => {
                return compare_argument_outlives(a, b);
            }
        })
    }
}

/// Takes two lists of arguments.
/// If they have any argument in common, the longer list outlives the shorter.
/// If they have no argument in common, it is not known whether one outlives
/// the other.
fn compare_argument_outlives(a: &[usize], b: &[usize]) -> Option<Ordering> {
    for &i in a {
        for &j in b {
            if i == j {
                return Some(a.len().cmp(&b.len()));
            }
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Assign,
    Set,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Fn,
    Block,
    Expr,
    Add,
    Mul,
    Pow,
    Base,
    Exp,
    Val,
    Call,
    Arg,
    Assign,
    Left,
    Right,
    Item,
    Return,
    Object,
    Array,
    ArrayItem,
    KeyValue,
    For,
    Init,
    Cond,
    Step,
    Compare,
    If,
    TrueBlock,
    ElseBlock,
    Loop,
    Id,
    Break,
    Continue,
    UnOp,
}

impl Kind {
    pub fn new(name: &str) -> Option<Kind> {
        Some(match name {
            "fn" => Kind::Fn,
            "block" => Kind::Block,
            "expr" => Kind::Expr,
            "add" => Kind::Add,
            "mul" => Kind::Mul,
            "pow" => Kind::Pow,
            "base" => Kind::Base,
            "exp" => Kind::Exp,
            "val" => Kind::Val,
            "call" => Kind::Call,
            "named_call" => Kind::Call,
            "arg" => Kind::Arg,
            "assign" => Kind::Assign,
            "left" => Kind::Left,
            "right" => Kind::Right,
            "item" => Kind::Item,
            "return" => Kind::Return,
            "object" => Kind::Object,
            "array" => Kind::Array,
            "array_item" => Kind::ArrayItem,
            "key_value" => Kind::KeyValue,
            "for" => Kind::For,
            "init" => Kind::Init,
            "cond" => Kind::Cond,
            "step" => Kind::Step,
            "compare" => Kind::Compare,
            "if" => Kind::If,
            "true_block" => Kind::TrueBlock,
            "else_block" => Kind::ElseBlock,
            "loop" => Kind::Loop,
            "id" => Kind::Id,
            "break" => Kind::Break,
            "continue" => Kind::Continue,
            "unop" => Kind::UnOp,
            _ => return None
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArgConstraint {
    Arg(usize),
    Return,
    Default,
}

#[derive(Debug, Copy, Clone)]
pub struct Intrinsic {
    pub arg_constraints: &'static [ArgConstraint],
    pub returns: bool,
}

static PRINTLN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static PRINT: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static CLONE: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static DEBUG: Intrinsic = Intrinsic {
    arg_constraints: &[],
    returns: false
};

static BACKTRACE: Intrinsic = Intrinsic {
    arg_constraints: &[],
    returns: false
};

static SLEEP: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static ROUND: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static RANDOM: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static READ_NUMBER: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static READ_LINE: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TRIM_RIGHT: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LEN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static PUSH: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default, ArgConstraint::Arg(0)],
    returns: false
};

static SQRT: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static ASIN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static SIN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static COS: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static ACOS: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TAN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static ATAN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static EXP: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOG2: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOG10: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TO_STRING: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};
