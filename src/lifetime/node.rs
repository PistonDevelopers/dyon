use super::kind::Kind;
use super::lt::{arg_lifetime, Lifetime};
use super::piston_meta::bootstrap::Convert;
use super::piston_meta::MetaData;
use super::ArgNames;
use ast::{AssignOp, BinOp};
use range::Range;
use std::sync::Arc;
use Lt;
use Type;

#[derive(Debug)]
pub(crate) struct Node {
    /// The kind of node.
    pub kind: Kind,
    /// The namespace alias.
    pub alias: Option<Arc<String>>,
    /// The names associated with a node.
    pub names: Vec<Arc<String>>,
    /// The type.
    pub ty: Option<Type>,
    /// Whether the argument or call argument is mutable.
    pub mutable: bool,
    /// Whether there is a `?` operator used on the node.
    pub try: bool,
    /// The grab level.
    pub grab_level: u16,
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
    pub op: Option<AssignOp>,
    /// Binary operators.
    pub binops: Vec<BinOp>,
    /// The argument lifetime constraints, one for each argument to a function.
    /// Just using an empty vector for nodes that are not functions.
    pub lts: Vec<Lt>,
}

impl Node {
    pub fn name(&self) -> Option<&Arc<String>> {
        if self.names.is_empty() {
            None
        } else {
            Some(&self.names[0])
        }
    }

    pub fn rewrite_unop(i: usize, name: Arc<String>, nodes: &mut [Node]) {
        nodes[i].kind = Kind::Call;
        nodes[i].names.push(name);
        let ch = nodes[i].children[0];
        nodes[ch].kind = Kind::CallArg;
    }

    pub fn rewrite_binop(i: usize, name: Arc<String>, nodes: &mut Vec<Node>) {
        nodes[i].kind = Kind::Call;
        nodes[i].names.push(name);
        nodes[i].binops.clear();

        let old_left = nodes[i].children[0];
        let old_right = nodes[i].children[1];

        let left = nodes.len();
        nodes.push(Node {
            kind: Kind::CallArg,
            names: vec![],
            ty: None,
            declaration: None,
            alias: None,
            mutable: false,
            try: false,
            grab_level: 0,
            source: nodes[old_left].source,
            start: nodes[old_left].start,
            end: nodes[old_left].end,
            lifetime: None,
            op: None,
            binops: vec![],
            lts: vec![],
            parent: Some(i),
            children: vec![old_left],
        });
        let right = nodes.len();
        nodes.push(Node {
            kind: Kind::CallArg,
            names: vec![],
            ty: None,
            declaration: None,
            alias: None,
            mutable: false,
            try: false,
            grab_level: 0,
            source: nodes[old_right].source,
            start: nodes[old_right].start,
            end: nodes[old_right].end,
            lifetime: None,
            op: None,
            binops: vec![],
            lts: vec![],
            parent: Some(i),
            children: vec![old_right],
        });

        nodes[old_left].parent = Some(left);
        nodes[old_right].parent = Some(right);

        nodes[i].children[0] = left;
        nodes[i].children[1] = right;
    }

    /// Simplifies a node by linking child with grand-parent.
    ///
    /// Removes the node that gets simplified.
    pub fn simplify(i: usize, nodes: &mut Vec<Node>) {
        if let Some(parent) = nodes[i].parent {
            // Link child to grand-parent.
            let ch = nodes[i].children[0];
            nodes[ch].parent = Some(parent);
            for p_ch in &mut nodes[parent].children {
                if *p_ch == i {
                    *p_ch = ch;
                }
            }

            // Disable this node.
            nodes[i].parent = None;
            nodes[i].children.clear();
        }
    }

    #[allow(dead_code)]
    pub fn print(&self, nodes: &[Node], indent: u32) {
        for _ in 0..indent {
            print!(" ")
        }
        println!(
            "kind: {:?}, name: {:?}, type: {:?}, decl: {:?} {{",
            self.kind,
            self.name(),
            self.ty,
            self.declaration
        );
        for &c in &self.children {
            nodes[c].print(nodes, indent + 1);
        }
        for _ in 0..indent {
            print!(" ")
        }
        println!("}}")
    }

    pub fn find_child_by_kind(&self, nodes: &[Node], kind: Kind) -> Option<usize> {
        for &ch in &self.children {
            if nodes[ch].kind == kind {
                return Some(ch);
            }
        }
        None
    }

    pub fn item_ids(&self) -> bool {
        self.kind == Kind::Item && !self.children.is_empty()
    }

    pub fn inner_type(&self, ty: &Type) -> Type {
        if self.try {
            match ty {
                &Type::Option(ref ty) => (**ty).clone(),
                &Type::Result(ref ty) => (**ty).clone(),
                x => x.clone(),
            }
        } else {
            ty.clone()
        }
    }

    pub fn has_lifetime(&self) -> bool {
        use super::kind::Kind::*;

        match self.kind {
            Pow | Sum | SumIn | Prod | ProdIn | SumVec4 | Min | MinIn | Max | MaxIn | Any
            | AnyIn | All | AllIn | LinkIn | Vec4 | Mat4 | Vec4UnLoop | Swizzle | Assign | For
            | ForN | ForIn | Link | LinkFor | Closure | CallClosure | Grab | TryExpr | Norm
            | In => false,
            Add | Mul | Compare => self.children.len() == 1,
            _ => true,
        }
    }

    pub fn lifetime(&self, nodes: &[Node], arg_names: &ArgNames) -> Option<Lifetime> {
        if !self.has_lifetime() {
            return None;
        }
        if let Some(declaration) = self.declaration {
            if self.kind == Kind::Item {
                let arg = &nodes[declaration];
                if arg.kind == Kind::Arg {
                    return arg_lifetime(declaration, arg, nodes, arg_names);
                } else if arg.kind == Kind::Current {
                    return Some(Lifetime::Current(declaration));
                } else {
                    return Some(Lifetime::Local(declaration));
                }
            }
        } else {
            // Intrinsic functions copies argument constraints to the call.
            if self.kind == Kind::Call && !self.lts.is_empty() {
                let mut returns_static = true;
                'args: for lt in self.lts.iter() {
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
                            Lt::Arg(x) => {
                                lt = self.lts[x];
                                continue;
                            }
                        }
                    }
                }

                if returns_static {
                    return None;
                }
            } else if self.kind == Kind::Item && self.name().map(|n| &**n == "return") == Some(true)
            {
                return Some(Lifetime::Return(vec![]));
            }
        }

        // Pick the smallest lifetime among children.
        let mut min: Option<Lifetime> = None;
        // TODO: Filter by kind of children.
        let mut call_arg_ind = 0;
        for &c in &self.children {
            match (self.kind, nodes[c].kind) {
                (_, Kind::Link) => {}
                (_, Kind::LinkFor) => {}
                (_, Kind::LinkIn) => {}
                (_, Kind::LinkItem) => {}
                (_, Kind::ReturnVoid) => {}
                (_, Kind::Swizzle) => {}
                (_, Kind::Loop) => {}
                (_, Kind::Go) => {}
                (_, Kind::For) => {}
                (_, Kind::ForN) => {}
                (_, Kind::ForIn) => {}
                (_, Kind::Break) => {}
                (_, Kind::Continue) => {}
                (_, Kind::Sift) => {}
                (_, Kind::SiftIn) => {}
                (_, Kind::Iter) => continue,
                (_, Kind::SumVec4) => {}
                (_, Kind::Sum) => {}
                (_, Kind::SumIn) => {}
                (_, Kind::Prod) => {}
                (_, Kind::ProdIn) => {}
                (_, Kind::ProdVec4) => {}
                (_, Kind::Min) => {}
                (_, Kind::MinIn) => {}
                (_, Kind::Max) => {}
                (_, Kind::MaxIn) => {}
                (_, Kind::Any) => {}
                (_, Kind::AnyIn) => {}
                (_, Kind::All) => {}
                (_, Kind::AllIn) => {}
                (_, Kind::Vec4UnLoop) => {}
                (_, Kind::Vec4) => {}
                (_, Kind::Mat4) => {}
                (_, Kind::Start) => continue,
                (_, Kind::End) => continue,
                (_, Kind::Assign) => {}
                (_, Kind::Object) => {}
                (_, Kind::KeyValue) => {}
                (_, Kind::Val) => {}
                (_, Kind::Add) => {}
                (_, Kind::Mul) => {}
                (_, Kind::Call) => {}
                (_, Kind::In) => {}
                (_, Kind::Closure) => {}
                (_, Kind::CallClosure) => {}
                (_, Kind::Grab) => {}
                (_, Kind::TryExpr) => {}
                (_, Kind::Arg) => continue,
                (_, Kind::Current) => continue,
                (Kind::CallClosure, Kind::Item) => continue,
                (_, Kind::Item) => {}
                (_, Kind::Norm) => {}
                (_, Kind::Compare) => {
                    // The result of all compare operators does not depend
                    // on the lifetime of the arguments.
                    continue;
                }
                (_, Kind::Left) => {}
                (_, Kind::Right) => {}
                (_, Kind::Expr) => {}
                (_, Kind::Return) => {}
                (_, Kind::Array) => {}
                (_, Kind::ArrayItem) => {}
                (_, Kind::ArrayFill) => {}
                (_, Kind::Pow) => {}
                (_, Kind::Block) => {}
                (_, Kind::If) => {}
                (_, Kind::TrueBlock) => {}
                (_, Kind::ElseIfBlock) => {}
                (_, Kind::ElseBlock) => {}
                (_, Kind::Cond) => {
                    // A condition controls the flow, but the result does not
                    // depend on its lifetime.
                    continue;
                }
                (_, Kind::ElseIfCond) => {
                    // A condition controls the flow, but the result does not
                    // depend on its lifetime.
                    continue;
                }
                (_, Kind::Fill) => {}
                (_, Kind::N) => {
                    // The result of array fill does not depend on `n`.
                    continue;
                }
                (Kind::Call, Kind::CallArg)
                | (Kind::CallClosure, Kind::CallArg)
                | (Kind::CallArg, Kind::CallArg) => {
                    // If there is no return lifetime on the declared argument,
                    // there is no need to check it, because the computed value
                    // does not depend on the lifetime of that argument.
                    if let Some(declaration) = self.declaration {
                        if let Some(&arg) = nodes[declaration]
                            .children
                            .iter()
                            .filter(|&&i| nodes[i].kind == Kind::Arg)
                            .nth(call_arg_ind)
                        {
                            match arg_lifetime(arg, &nodes[arg], nodes, arg_names) {
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
                x => panic!(
                    "Unimplemented `{:?}`. \
                        Perhaps you need add something to `Node::has_lifetime`?",
                    x
                ),
            }
            let lifetime = match nodes[c].lifetime(nodes, arg_names) {
                Some(lifetime) => lifetime,
                None => {
                    continue;
                }
            };
            if min.is_none() || min.as_ref().map(|l| l < &lifetime) == Some(true) {
                min = Some(lifetime);
            }
        }
        min
    }
}

pub(crate) fn convert_meta_data(
    nodes: &mut Vec<Node>,
    data: &[Range<MetaData>],
) -> Result<(), Range<String>> {
    let mut parents: Vec<usize> = vec![];
    let ignored = &mut vec![];
    let mut skip: Option<usize> = None;
    for (i, d) in data.iter().enumerate() {
        if let Some(j) = skip {
            if j > i {
                continue;
            }
        }
        match d.data {
            MetaData::StartNode(ref kind_name) => {
                let kind = match Kind::new(kind_name) {
                    Some(kind) => kind,
                    None => return Err(d.range().wrap(format!("Unknown kind `{}`", kind_name))),
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

                let ty = match kind {
                    Kind::Array | Kind::ArrayFill => Some(Type::array()),
                    Kind::Vec4 | Kind::Vec4UnLoop => Some(Type::Vec4),
                    Kind::Mat4 => Some(Type::Mat4),
                    Kind::EX | Kind::EY | Kind::EZ | Kind::EW => Some(Type::Vec4),
                    Kind::In => Some(Type::In(Box::new(Type::array()))),
                    Kind::Object => Some(Type::object()),
                    Kind::Sift | Kind::SiftIn => Some(Type::array()),
                    Kind::Sum | Kind::SumIn | Kind::Prod | Kind::ProdIn => Some(Type::F64),
                    Kind::Swizzle => Some(Type::F64),
                    Kind::Link | Kind::LinkFor => Some(Type::Link),
                    Kind::Any | Kind::AnyIn | Kind::All | Kind::AllIn => {
                        Some(Type::Secret(Box::new(Type::Bool)))
                    }
                    Kind::Min | Kind::MinIn | Kind::Max | Kind::MaxIn => {
                        Some(Type::Secret(Box::new(Type::F64)))
                    }
                    Kind::For | Kind::ForN => Some(Type::Void),
                    Kind::TyArg | Kind::TyRet => {
                        // Parse extra type information.
                        let convert = Convert::new(&data[i..]);
                        if let Ok((range, val)) = Type::from_meta_data(kind_name, convert, ignored)
                        {
                            // Skip content of type meta data until end node.
                            skip = Some(range.next_offset() + i - 1);
                            Some(val)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                let parent = parents.last().copied();
                parents.push(nodes.len());
                nodes.push(Node {
                    kind,
                    alias: None,
                    names: vec![],
                    ty,
                    mutable: false,
                    try: false,
                    grab_level: 0,
                    source: Range::empty(0),
                    parent,
                    children: vec![],
                    start: i,
                    end: 0,
                    lifetime: None,
                    declaration: None,
                    op: None,
                    binops: vec![],
                    lts: vec![],
                });
            }
            MetaData::EndNode(_) => {
                let ind = parents.pop().unwrap();
                {
                    let node = &mut nodes[ind];
                    node.source = d.range();
                    node.end = i + 1;
                }
                if let Some(&parent) = parents.last() {
                    nodes[parent].children.push(ind);
                }
            }
            MetaData::String(ref n, ref val) => {
                match &***n {
                    "alias" => {
                        let i = *parents.last().unwrap();
                        nodes[i].alias = Some(val.clone());
                    }
                    "name" => {
                        let i = *parents.last().unwrap();
                        nodes[i].names.push(val.clone());
                    }
                    "word" => {
                        // Put words together to name.
                        let i = *parents.last().unwrap();
                        if nodes[i].names.is_empty() {
                            let mut name = val.clone();
                            if nodes[i].kind != Kind::CallClosure {
                                Arc::make_mut(&mut name).push('_');
                            }
                            nodes[i].names.push(name);
                        } else if let Some(ref mut name) = nodes[i].names.get_mut(0) {
                            let name = Arc::make_mut(name);
                            name.push('_');
                            name.push_str(val);
                        }
                    }
                    "lifetime" => {
                        let i = *parents.last().unwrap();
                        nodes[i].lifetime = Some(val.clone());
                    }
                    "text" => {
                        let i = *parents.last().unwrap();
                        nodes[i].ty = Some(Type::Str);
                    }
                    "color" => {
                        let i = *parents.last().unwrap();
                        nodes[i].ty = Some(Type::Vec4);
                    }
                    "ty_var" => {
                        // Use names as a way of storing type variables.
                        let i = *parents.last().unwrap();
                        nodes[i].names.push(val.clone());
                    }
                    _ => {}
                }
            }
            MetaData::Bool(ref n, _val) => {
                match &***n {
                    ":=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].op = Some(AssignOp::Assign);
                    }
                    "=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].op = Some(AssignOp::Set);
                    }
                    "+=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].op = Some(AssignOp::Add);
                    }
                    "-=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].op = Some(AssignOp::Sub);
                    }
                    "*=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].op = Some(AssignOp::Mul);
                    }
                    "/=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].op = Some(AssignOp::Div);
                    }
                    "%=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].op = Some(AssignOp::Rem);
                    }
                    "^=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].op = Some(AssignOp::Pow);
                    }
                    "mut" => {
                        let i = *parents.last().unwrap();
                        nodes[i].mutable = _val;
                    }
                    "try" | "try_item" => {
                        let i = *parents.last().unwrap();
                        nodes[i].try = _val;
                    }
                    "bool" => {
                        let i = *parents.last().unwrap();
                        nodes[i].ty = Some(Type::Bool);
                    }
                    "returns" => {
                        // Assuming this will be overwritten when
                        // type is parsed or inferred.
                        let i = *parents.last().unwrap();
                        if _val {
                            nodes[i].ty = Some(Type::Any);
                        } else {
                            nodes[i].ty = Some(Type::Void);
                        }
                    }
                    "return_void" => {
                        // There is no sub node, so we need change kind of parent.
                        // This should always be an expression.
                        let i = *parents.last().unwrap();
                        nodes[i].kind = Kind::ReturnVoid;
                    }
                    "*." => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::Dot);
                    }
                    "x" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::Cross);
                    }
                    "*" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::Mul);
                    }
                    "/" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::Div);
                    }
                    "%" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::Rem);
                    }
                    "^" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::Pow);
                    }
                    "&&" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::AndAlso);
                    }
                    "+" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::Add);
                    }
                    "-" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::Sub);
                    }
                    "||" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::OrElse);
                    }
                    "<" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::Less);
                    }
                    "<=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::LessOrEqual);
                    }
                    ">" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::Greater);
                    }
                    ">=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::GreaterOrEqual);
                    }
                    "==" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::Equal);
                    }
                    "!=" => {
                        let i = *parents.last().unwrap();
                        nodes[i].binops.push(BinOp::NotEqual);
                    }
                    _ => {}
                }
            }
            MetaData::F64(ref n, val) => match &***n {
                "num" => {
                    let i = *parents.last().unwrap();
                    nodes[i].ty = Some(Type::F64);
                }
                "grab_level" => {
                    if val < 1.0 {
                        return Err(d
                            .range()
                            .wrap("Grab level must be at least `'1`".to_string()));
                    }
                    let i = *parents.last().unwrap();
                    nodes[i].grab_level = val as u16;
                }
                _ => {}
            },
        }
    }
    Ok(())
}
