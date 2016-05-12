use std::sync::Arc;
use range::Range;
use super::piston_meta::MetaData;
use super::piston_meta::bootstrap::Convert;
use super::lt::{arg_lifetime, Lifetime};
use super::kind::Kind;
use super::Op;
use super::ArgNames;
use Lt;
use Type;

#[derive(Debug)]
pub struct Node {
    /// The kind of node.
    pub kind: Kind,
    /// The name.
    pub name: Option<Arc<String>>,
    /// The type.
    pub ty: Option<Type>,
    /// Whether the argument or call argument is mutable.
    pub mutable: bool,
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
    /// The argument lifetime constraints, one for each argument to a function.
    /// Just using an empty vector for nodes that are not functions.
    pub lts: Vec<Lt>,
}

impl Node {
    pub fn print(&self, nodes: &[Node], indent: u32) {
        for _ in 0..indent { print!(" ") }
        println!("kind: {:?}, name: {:?}, type: {:?} {{", self.kind, self.name, self.ty);
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
        match self.kind {
            Kind::Add | Kind::Mul | Kind::Pow | Kind::Compare
            | Kind::Sum | Kind::Min | Kind::Max | Kind::Any | Kind::All
            | Kind::Vec4 => {
                if self.children.len() > 1 {
                    return None;
                }
            }
            _ => {}
        }
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
            if self.kind == Kind::Call && self.lts.len() > 0 {
                let mut returns_static = true;
                'args: for (_, lt) in self.children.iter().map(|&i| &nodes[i])
                        .filter(|&n| n.kind == Kind::CallArg)
                        .zip(self.lts.iter()) {
                    let mut lt = *lt;
                    loop {
                        match lt {
                            Lt::Default => {
                                continue 'args;
                            }
                            Lt::Return => {
                                returns_static = false;
                                break 'args;
                            }
                            x => {
                                lt = x;
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

        // Pick the smallest lifetime among children.
        let mut min: Option<Lifetime> = None;
        // TODO: Filter by kind of children.
        let mut call_arg_ind = 0;
        for &c in &self.children {
            match (self.kind, nodes[c].kind) {
                (_, Kind::ForN) => {}
                (_, Kind::Continue) => {}
                (_, Kind::Sift) => {}
                (_, Kind::Sum) => {}
                (_, Kind::Min) => {}
                (_, Kind::Max) => {}
                (_, Kind::Any) => {}
                (_, Kind::All) => {}
                (_, Kind::Vec4) => {}
                (_, Kind::Start) => { continue }
                (_, Kind::End) => { continue }
                (_, Kind::Assign) => {}
                (_, Kind::Object) => {}
                (_, Kind::KeyValue) => {}
                (_, Kind::Val) => {}
                (_, Kind::Add) => {}
                (_, Kind::Mul) => {}
                (_, Kind::Call) => {}
                (_, Kind::Item) => {}
                (_, Kind::UnOp) => {
                    // The result of all unary operators does not depend
                    // on the lifetime of the argument.
                    continue
                }
                (_, Kind::Compare) => {
                    // The result of all compare operators does not depend
                    // on the lifetime of the arguments.
                    continue
                }
                (_, Kind::Left) => {}
                (_, Kind::Right) => {}
                (_, Kind::Expr) => {}
                (_, Kind::Return) => {}
                (_, Kind::Array) => {}
                (_, Kind::ArrayItem) => {}
                (_, Kind::ArrayFill) => {}
                (_, Kind::Pow) => {}
                (_, Kind::Base) => {}
                (_, Kind::Exp) => {}
                (_, Kind::Block) => {}
                (_, Kind::If) => {}
                (_, Kind::TrueBlock) => {}
                (_, Kind::ElseIfBlock) => {}
                (_, Kind::ElseBlock) => {}
                (_, Kind::Cond) => {
                    // A condition controls the flow, but the result does not
                    // depend on its lifetime.
                    continue
                }
                (_, Kind::ElseIfCond) => {
                    // A condition controls the flow, but the result does not
                    // depend on its lifetime.
                    continue
                }
                (_, Kind::Fill) => {}
                (_, Kind::N) => {
                    // The result of array fill does not depend on `n`.
                    continue
                }
                (Kind::Call, Kind::CallArg) => {
                    // If there is no return lifetime on the declared argument,
                    // there is no need to check it, because the computed value
                    // does not depend on the lifetime of that argument.
                    if let Some(declaration) = self.declaration {
                        if let Some(&arg) = nodes[declaration].children.iter()
                            .filter(|&&i| nodes[i].kind == Kind::Arg)
                            .nth(call_arg_ind) {
                            match arg_lifetime(arg, &nodes[arg],
                                               nodes, arg_names) {
                                Some(Lifetime::Return(_)) => {}
                                _ => {
                                    call_arg_ind += 1;
                                    continue;
                                }
                            }
                        }
                    }
                    call_arg_ind += 1;
                }
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

pub fn convert_meta_data(
    nodes: &mut Vec<Node>,
    data: &[Range<MetaData>]
) -> Result<(), Range<String>> {
    let mut parents: Vec<usize> = vec![];
    let ref mut ignored = vec![];
    let mut skip: Option<usize> = None;
    for (i, d) in data.iter().enumerate() {
        if let Some(j) = skip {
            if j > i { continue; }
        }
        match d.data {
            MetaData::StartNode(ref kind_name) => {
                let kind = match Kind::new(kind_name) {
                    Some(kind) => kind,
                    None => return Err(d.range().wrap(format!("Unknown kind `{}`", kind_name)))
                };

                // Parse type information and put it in parent node.
                if kind == Kind::Type || kind == Kind::RetType {
                    let convert = Convert::new(&data[i..]);
                    if let Ok((range, val)) = Type::from_meta_data(kind_name, convert, ignored) {
                        let parent = *parents.last().unwrap();
                        nodes[parent].ty = Some(val);
                        skip = Some(range.next_offset() + i);
                        continue;
                    }
                }

                if kind == Kind::Expr {
                    let parent = *parents.last().unwrap();
                    if nodes[parent].kind == Kind::Fn {
                        // Function returns a value.
                        nodes[parent].ty = Some(Type::Any);
                    }
                }

                let ty = match kind {
                    Kind::Fn => Some(Type::Void),
                    Kind::Array | Kind::ArrayFill => Some(Type::array()),
                    Kind::Vec4 => Some(Type::Vec4),
                    Kind::Object => Some(Type::object()),
                    _ => None
                };

                let parent = parents.last().map(|i| *i);
                parents.push(nodes.len());
                nodes.push(Node {
                    kind: kind,
                    name: None,
                    ty: ty,
                    mutable: false,
                    source: Range::empty(0),
                    parent: parent,
                    children: vec![],
                    start: i,
                    end: 0,
                    lifetime: None,
                    declaration: None,
                    op: None,
                    ids: 0,
                    lts: vec![]
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
                    "text" => {
                        let i = *parents.last().unwrap();
                        nodes[i].ty = Some(Type::Text);
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
                    "mut" => {
                        let i = *parents.last().unwrap();
                        nodes[i].mutable = _val;
                    }
                    "bool" => {
                        let i = *parents.last().unwrap();
                        nodes[i].ty = Some(Type::Bool);
                    }
                    "returns" => {
                        // Assuming this will be overwritten when
                        // type is parsed or inferred.
                        let i = *parents.last().unwrap();
                        nodes[i].ty = Some(Type::Any);
                    }
                    "return_void" => {
                        // There is no sub node, so we need change kind of parent.
                        // This should always be an expression.
                        let i = *parents.last().unwrap();
                        nodes[i].kind = Kind::ReturnVoid;
                    }
                    _ => {}
                }
            }
            MetaData::F64(ref n, _) => {
                match &***n {
                    "num" => {
                        let i = *parents.last().unwrap();
                        nodes[i].ty = Some(Type::F64);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
