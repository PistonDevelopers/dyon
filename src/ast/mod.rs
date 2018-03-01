use std::collections::HashMap;
use std::sync::Arc;
use std::cell::Cell;
use range::Range;
use piston_meta::bootstrap::Convert;
use piston_meta::MetaData;

use FnIndex;
use Module;
use Prelude;
use Type;
use Variable;

mod infer_len;
mod replace;

pub fn convert(
    file: Arc<String>,
    source: Arc<String>,
    data: &[Range<MetaData>],
    ignored: &mut Vec<Range>,
    module: &mut Module
) -> Result<(), ()> {
    let mut convert = Convert::new(data);

    let namespace = if let Ok((range, val)) = Namespace::from_meta_data(convert, ignored) {
        convert.update(range);
        val.names
    } else {
        Arc::new(vec![])
    };

    let use_lookup = if let Ok((range, val)) = Uses::from_meta_data(convert, ignored) {
        convert.update(range);
        UseLookup::from_uses_module(&val, module)
    } else {
        UseLookup::new()
    };

    loop {
        if let Ok((range, function)) =
        Function::from_meta_data(&namespace, &file, &source, "fn", convert, ignored) {
            convert.update(range);
            module.register(function);
        } else if convert.remaining_data_len() > 0 {
            return Err(());
        } else {
            break;
        }
    }
    for (i, f) in module.functions.iter().enumerate() {
        f.resolve_locals(i, module, &use_lookup);
    }
    Ok(())
}

/// Used to resolve calls to imported functions.
pub struct UseLookup {
    pub aliases: HashMap<Arc<String>, HashMap<Arc<String>, usize>>,
}

impl UseLookup {
    pub fn new() -> UseLookup {
        UseLookup {
            aliases: HashMap::new(),
        }
    }

    pub fn from_uses_module(uses: &Uses, module: &Module) -> UseLookup {
        let mut aliases = HashMap::new();
        // First, add all glob imports.
        for use_import in &uses.use_imports {
            if use_import.fns.len() > 0 {continue;}
            if !aliases.contains_key(&use_import.alias) {
                aliases.insert(use_import.alias.clone(), HashMap::new());
            }
            let fns = aliases.get_mut(&use_import.alias).unwrap();
            for (i, f) in module.functions.iter().enumerate().rev() {
                if &*f.namespace == &use_import.names {
                    fns.insert(f.name.clone(), i);
                }
            }
        }
        // Second, add specific functions, which shadows glob imports.
        for use_import in &uses.use_imports {
            if use_import.fns.len() == 0 {continue;}
            if !aliases.contains_key(&use_import.alias) {
                aliases.insert(use_import.alias.clone(), HashMap::new());
            }
            let fns = aliases.get_mut(&use_import.alias).unwrap();
            for use_fn in &use_import.fns {
                for (i, f) in module.functions.iter().enumerate().rev() {
                    if &*f.namespace != &use_import.names {continue;}
                    if &f.name == &use_fn.0 {
                        fns.insert(use_fn.1.as_ref().unwrap_or(&use_fn.0).clone(), i);
                    } else if f.name.len() > use_fn.0.len() &&
                              f.name.starts_with(&**use_fn.0) &&
                              f.name.as_bytes()[use_fn.0.len()] == '(' as u8 {
                        // A function with mutable information.
                        let mut name: Arc<String> = use_fn.1.as_ref().unwrap_or(&use_fn.0).clone();
                        Arc::make_mut(&mut name).push_str(&f.name.as_str()[use_fn.0.len()..]);
                        fns.insert(name, i);
                    }
                }
            }
        }
        UseLookup {
            aliases: aliases,
        }
    }

    pub fn from_uses_prelude(uses: &Uses, prelude: &Prelude) -> UseLookup {
        let mut aliases = HashMap::new();
        // First, add all glob imports.
        for use_import in &uses.use_imports {
            if use_import.fns.len() > 0 {continue;}
            if !aliases.contains_key(&use_import.alias) {
                aliases.insert(use_import.alias.clone(), HashMap::new());
            }
            let fns = aliases.get_mut(&use_import.alias).unwrap();
            for (i, f) in prelude.namespaces.iter().enumerate().rev() {
                if &*f.0 == &use_import.names {
                    fns.insert(f.1.clone(), i);
                }
            }
        }
        // Second, add specific functions, which shadows glob imports.
        for use_import in &uses.use_imports {
            if use_import.fns.len() == 0 {continue;}
            if !aliases.contains_key(&use_import.alias) {
                aliases.insert(use_import.alias.clone(), HashMap::new());
            }
            let fns = aliases.get_mut(&use_import.alias).unwrap();
            for use_fn in &use_import.fns {
                for (i, f) in prelude.namespaces.iter().enumerate().rev() {
                    if &*f.0 != &use_import.names {continue;}
                    if &f.1 == &use_fn.0 {
                        fns.insert(use_fn.1.as_ref().unwrap_or(&use_fn.0).clone(), i);
                    } else if f.1.len() > use_fn.0.len() &&
                              f.1.starts_with(&**use_fn.0) &&
                              f.1.as_bytes()[use_fn.0.len()] == '(' as u8 {
                        // A function with mutable information.
                        let mut name: Arc<String> = use_fn.1.as_ref().unwrap_or(&use_fn.0).clone();
                        Arc::make_mut(&mut name).push_str(&f.1.as_str()[use_fn.0.len()..]);
                        fns.insert(name, i);
                    }
                }
            }
        }
        UseLookup {
            aliases: aliases,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Namespace {
    pub names: Arc<Vec<Arc<String>>>,
}

impl Namespace {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Namespace), ()> {
        let start = convert.clone();
        let node = "ns";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut names: Vec<Arc<String>> = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                names.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), Namespace {
            names: Arc::new(names)
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Uses {
    pub use_imports: Vec<UseImport>,
}

impl Uses {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Uses), ()> {
        let start = convert.clone();
        let node = "uses";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut use_imports = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = UseImport::from_meta_data(convert, ignored) {
                convert.update(range);
                use_imports.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), Uses {
            use_imports: use_imports
        }))
    }
}

#[derive(Debug, Clone)]
pub struct UseImport {
    pub names: Vec<Arc<String>>,
    pub fns: Vec<(Arc<String>, Option<Arc<String>>)>,
    pub alias: Arc<String>,
}

impl UseImport {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, UseImport), ()> {
        let start = convert.clone();
        let node = "use";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut names: Vec<Arc<String>> = vec![];
        let mut alias: Option<Arc<String>> = None;
        let mut fns = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                names.push(val);
            } else if let Ok((range, val)) = convert.meta_string("use_fn") {
                convert.update(range);

                let fn_alias = if let Ok((range, val)) = convert.meta_string("use_fn_alias") {
                    convert.update(range);
                    Some(val)
                } else {
                    None
                };
                fns.push((val, fn_alias));
            } else if let Ok((range, val)) = convert.meta_string("alias") {
                convert.update(range);
                alias = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let alias = alias.ok_or(())?;
        Ok((convert.subtract(start), UseImport {
            names: names,
            fns: fns,
            alias: alias,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub namespace: Arc<Vec<Arc<String>>>,
    pub name: Arc<String>,
    pub file: Arc<String>,
    pub source: Arc<String>,
    pub args: Vec<Arg>,
    pub currents: Vec<Current>,
    pub block: Block,
    pub ret: Type,
    pub resolved: Arc<::std::sync::atomic::AtomicBool>,
    pub source_range: Range,
    pub senders: Arc<(
        ::std::sync::atomic::AtomicBool,
        ::std::sync::Mutex<Vec<::std::sync::mpsc::Sender<Variable>>>
    )>,
}

impl Function {
    pub fn from_meta_data(
        namespace: &Arc<Vec<Arc<String>>>,
        file: &Arc<String>,
        source: &Arc<String>,
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Function), ()> {
        use std::sync::Mutex;
        use std::sync::atomic::AtomicBool;

        let start = convert.clone();
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
        let mut args: Vec<Arg> = vec![];
        let mut currents: Vec<Current> = vec![];
        let mut block: Option<Block> = None;
        let mut expr: Option<Expression> = None;
        let mut ret: Option<Type> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else if let Ok((range, val)) = Arg::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                args.push(val);
            } else if let Ok((range, val)) = Current::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                currents.push(val);
            } else if let Ok((range, val)) = convert.meta_bool("returns") {
                convert.update(range);
                ret = Some(if val { Type::Any } else { Type::Void })
            } else if let Ok((range, val)) = Type::from_meta_data(
                    "ret_type", convert, ignored) {
                convert.update(range);
                ret = Some(val);
            } else if let Ok((range, val)) = Block::from_meta_data(
                    file, source, "block", convert, ignored) {
                convert.update(range);
                block = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "expr", convert, ignored) {
                convert.update(range);
                expr = Some(val);
                ret = Some(Type::Any);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let mut name = try!(name.ok_or(()));
        let block = match expr {
            None => try!(block.ok_or(())),
            Some(expr) => {
                let source_range = expr.source_range();
                Block {
                    expressions: vec![Expression::Return(Box::new(expr))],
                    source_range: source_range
                }
            }
        };
        let mutable_args = args.iter().any(|arg| arg.mutable);
        if mutable_args {
            let mut name_plus_args = String::from(&**name);
            name_plus_args.push('(');
            let mut first = true;
            for arg in &args {
                if !first { name_plus_args.push(','); }
                name_plus_args.push_str(if arg.mutable { "mut" } else { "_" });
                first = false;
            }
            name_plus_args.push(')');
            name = Arc::new(name_plus_args);
        }
        let ret = try!(ret.ok_or(()));
        Ok((convert.subtract(start), Function {
            namespace: namespace.clone(),
            resolved: Arc::new(AtomicBool::new(false)),
            name: name,
            file: file.clone(),
            source: source.clone(),
            args: args,
            currents: currents,
            block: block,
            ret: ret,
            source_range: convert.source(start).unwrap(),
            senders: Arc::new((AtomicBool::new(false), Mutex::new(vec![]))),
        }))
    }

    pub fn returns(&self) -> bool { self.ret != Type::Void }

    pub fn resolve_locals(&self, relative: usize, module: &Module, use_lookup: &UseLookup) {
        use std::sync::atomic::Ordering;

        // Ensure sequential order just to be safe.
        if self.resolved.load(Ordering::SeqCst) { return; }
        let mut stack: Vec<Option<Arc<String>>> = vec![];
        let mut closure_stack: Vec<usize> = vec![];
        if self.returns() {
            stack.push(Some(Arc::new("return".into())));
        }
        for arg in &self.args {
            stack.push(Some(arg.name.clone()));
        }
        for current in &self.currents {
            stack.push(Some(current.name.clone()));
        }
        self.block.resolve_locals(relative, &mut stack, &mut closure_stack, module, use_lookup);
        self.resolved.store(true, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone)]
pub struct Closure {
    pub file: Arc<String>,
    pub source: Arc<String>,
    pub args: Vec<Arg>,
    pub currents: Vec<Current>,
    pub expr: Expression,
    pub ret: Type,
    pub source_range: Range,
}

impl Closure {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Closure), ()> {
        let start = convert.clone();
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut args: Vec<Arg> = vec![];
        let mut currents: Vec<Current> = vec![];
        let mut expr: Option<Expression> = None;
        let mut ret: Option<Type> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Arg::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                args.push(val);
            } else if let Ok((range, val)) = Current::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                currents.push(val);
            } else if let Ok((range, val)) = convert.meta_bool("returns") {
                convert.update(range);
                ret = Some(if val { Type::Any } else { Type::Void })
            } else if let Ok((range, val)) = Type::from_meta_data(
                    "ret_type", convert, ignored) {
                convert.update(range);
                ret = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "expr", convert, ignored) {
                convert.update(range);
                expr = Some(val);
                ret = Some(Type::Any);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let ret = try!(ret.ok_or(()));
        let expr = try!(expr.ok_or(()));
        Ok((convert.subtract(start), Closure {
            file: file.clone(),
            source: source.clone(),
            args: args,
            currents: currents,
            expr: expr,
            ret: ret,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn returns(&self) -> bool { self.ret != Type::Void }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        // Start new closure.
        // Need a closure stack because `grab` expressions are relative
        // to closure environment, not the locals inside the closure.
        let cs = closure_stack.len();
        closure_stack.push(stack.len());
        if self.returns() {
            stack.push(Some(Arc::new("return".into())));
        }
        for arg in &self.args {
            stack.push(Some(arg.name.clone()));
        }
        for current in &self.currents {
            stack.push(Some(current.name.clone()));
        }
        self.expr.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        closure_stack.truncate(cs);
    }
}

#[derive(Debug, Clone)]
pub struct Grab {
    pub level: u16,
    pub expr: Expression,
    pub source_range: Range,
}

impl Grab {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Grab), ()> {
        let start = convert.clone();
        let node = "grab";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut level: Option<u16> = None;
        let mut expr: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_f64("grab_level") {
                convert.update(range);
                level = Some(val as u16);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "expr", convert, ignored) {
                convert.update(range);
                expr = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let level = level.unwrap_or(1);
        let expr = try!(expr.ok_or(()));
        Ok((convert.subtract(start), Grab {
            level: level,
            expr: expr,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn precompute(&self) -> Option<Variable> {
        self.expr.precompute()
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        // Get closure environment.
        let d = if closure_stack.len() < self.level as usize {
                // Ignore negative difference, because
                // lifetime checker will detect too high grab level.
                0
            } else {
                closure_stack.len() - self.level as usize
            };
        let last = match closure_stack.get(d) {
            None => return,
            Some(&x) => x,
        };
        // Use environment outside closure.
        let mut tmp_stack: Vec<_> = stack[0..last].into();
        self.expr.resolve_locals(relative, &mut tmp_stack, closure_stack, module, use_lookup)
    }
}

#[derive(Debug, Clone)]
pub struct TryExpr {
    pub expr: Expression,
    pub source_range: Range,
}

impl TryExpr {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, TryExpr), ()> {
        let start = convert.clone();
        let node = "try_expr";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut expr: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "expr", convert, ignored) {
                convert.update(range);
                expr = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let expr = try!(expr.ok_or(()));
        Ok((convert.subtract(start), TryExpr {
            expr: expr,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        self.expr.resolve_locals(relative, stack, closure_stack, module, use_lookup)
    }
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: Arc<String>,
    pub lifetime: Option<Arc<String>>,
    pub ty: Type,
    pub source_range: Range,
    pub mutable: bool,
}

impl Arg {
    pub fn from_meta_data(mut convert: Convert, ignored: &mut Vec<Range>)
    -> Result<(Range, Arg), ()> {
        let start = convert.clone();
        let node = "arg";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
        let mut lifetime: Option<Arc<String>> = None;
        let mut ty: Option<Type> = None;
        let mut mutable = false;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_bool("mut") {
                convert.update(range);
                mutable = val;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("lifetime") {
                convert.update(range);
                lifetime = Some(val);
            } else if let Ok((range, val)) = Type::from_meta_data(
                    "type", convert, ignored) {
                convert.update(range);
                ty = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let name = try!(name.ok_or(()));
        let ty = match ty {
            None => Type::Any,
            Some(ty) => ty
        };
        Ok((convert.subtract(start), Arg {
            name: name,
            lifetime: lifetime,
            ty: ty,
            source_range: convert.source(start).unwrap(),
            mutable: mutable,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Current {
    pub name: Arc<String>,
    pub source_range: Range,
    pub mutable: bool,
}

impl Current {
    pub fn from_meta_data(mut convert: Convert, ignored: &mut Vec<Range>)
    -> Result<(Range, Current), ()> {
        let start = convert.clone();
        let node = "current";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
        let mut mutable = false;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_bool("mut") {
                convert.update(range);
                mutable = val;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else if let Ok((range, _)) = Type::from_meta_data(
                    "type", convert, ignored) {
                convert.update(range);
                // Just ignore type for now.
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let name = try!(name.ok_or(()));
        Ok((convert.subtract(start), Current {
            name: name,
            source_range: convert.source(start).unwrap(),
            mutable: mutable,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub expressions: Vec<Expression>,
    pub source_range: Range,
}

impl Block {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Block), ()> {
        let start = convert.clone();
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut expressions = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "expr", convert, ignored) {
                convert.update(range);
                expressions.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), Block {
            expressions: expressions,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for expr in &self.expressions {
            expr.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        }
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Link(Link),
    Object(Box<Object>),
    Array(Box<Array>),
    ArrayFill(Box<ArrayFill>),
    Return(Box<Expression>),
    ReturnVoid(Range),
    Break(Break),
    Continue(Continue),
    Block(Block),
    Go(Box<Go>),
    // TODO: Check size, perhaps use `Box<Call>`?
    Call(Call),
    Item(Item),
    BinOp(Box<BinOpExpression>),
    Assign(Box<Assign>),
    Vec4(Vec4),
    For(Box<For>),
    ForN(Box<ForN>),
    Sum(Box<ForN>),
    SumVec4(Box<ForN>),
    Prod(Box<ForN>),
    ProdVec4(Box<ForN>),
    Min(Box<ForN>),
    Max(Box<ForN>),
    Sift(Box<ForN>),
    Any(Box<ForN>),
    All(Box<ForN>),
    LinkFor(Box<ForN>),
    If(Box<If>),
    Compare(Box<Compare>),
    UnOp(Box<UnOpExpression>),
    Norm(Box<Norm>),
    Variable(Range, Variable),
    Try(Box<Expression>),
    Swizzle(Box<Swizzle>),
    Closure(Arc<Closure>),
    CallClosure(Box<CallClosure>),
    Grab(Box<Grab>),
    TryExpr(Box<TryExpr>),
    In(Box<In>),
}

// Required because the `Sync` impl of `Variable` is unsafe.
unsafe impl Sync for Expression {}

impl Expression {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Expression), ()> {
        let start = convert.clone();
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut result: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, _)) = convert.meta_bool("mut") {
                // Ignore `mut` since it is handled by lifetime checker.
                convert.update(range);
            } else if let Ok((range, val)) = Link::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Link(val));
            } else if let Ok((range, val)) = Object::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Object(Box::new(val)));
            } else if let Ok((range, val)) = Array::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Array(Box::new(val)));
            } else if let Ok((range, val)) = ArrayFill::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::ArrayFill(Box::new(val)));
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "return", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Return(Box::new(val)));
            } else if let Ok((range, _)) = convert.meta_bool("return_void") {
                convert.update(range);
                result = Some(Expression::ReturnVoid(
                    convert.source(start).unwrap()));
            } else if let Ok((range, val)) = Break::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Break(val));
            } else if let Ok((range, val)) = Continue::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Continue(val));
            } else if let Ok((range, val)) = Block::from_meta_data(
                    file, source, "block", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Block(val));
            } else if let Ok((range, val)) = Add::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(val.to_expression());
            } else if let Ok((range, val)) = UnOpExpression::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::UnOp(Box::new(val)));
            } else if let Ok((range, val)) = Mul::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(val.to_expression());
            } else if let Ok((range, val)) = Item::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Item(val));
            } else if let Ok((range, val)) = Norm::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Norm(Box::new(val)));
            } else if let Ok((range, val)) = convert.meta_string("text") {
                convert.update(range);
                result = Some(Expression::Variable(
                    convert.source(start).unwrap(),
                    Variable::Text(val)
                ));
            } else if let Ok((range, val)) = convert.meta_f64("num") {
                convert.update(range);
                result = Some(Expression::Variable(
                    convert.source(start).unwrap(),
                    Variable::f64(val)
                ));
            } else if let Ok((range, val)) = Vec4::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Vec4(val));
            } else if let Ok((range, val)) = Vec4UnLoop::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(val.to_expression());
            } else if let Ok((range, val)) = convert.meta_bool("bool") {
                convert.update(range);
                result = Some(Expression::Variable(
                    convert.source(start).unwrap(), Variable::bool(val)
                ));
            } else if let Ok((range, val)) = convert.meta_string("color") {
                use read_color;

                convert.update(range);
                if let Some((rgb, a)) = read_color::rgb_maybe_a(&mut val.chars()) {
                    let v = [rgb[0] as f32 / 255.0, rgb[1] as f32 / 255.0, rgb[2] as f32 / 255.0,
                             a.unwrap_or(255) as f32 / 255.0];
                    result = Some(Expression::Variable(range, Variable::Vec4(v)));
                } else {
                    return Err(());
                }
            } else if let Ok((range, val)) = Go::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Go(Box::new(val)));
            } else if let Ok((range, val)) = Call::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Call(val));
            } else if let Ok((range, val)) = Call::named_from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Call(val));
            } else if let Ok((range, val)) = Assign::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Assign(Box::new(val)));
            } else if let Ok((range, val)) = For::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::For(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "for_n", convert, ignored) {
                convert.update(range);
                result = Some(Expression::ForN(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "sum", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Sum(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "sum_vec4", convert, ignored) {
                convert.update(range);
                result = Some(Expression::SumVec4(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "prod", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Prod(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "prod_vec4", convert, ignored) {
                convert.update(range);
                result = Some(Expression::ProdVec4(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "min", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Min(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "max", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Max(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "sift", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Sift(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "any", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Any(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "all", convert, ignored) {
                convert.update(range);
                result = Some(Expression::All(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "link_for", convert, ignored) {
                convert.update(range);
                result = Some(Expression::LinkFor(Box::new(val)));
            } else if let Ok((range, val)) = Loop::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(val.to_expression());
            } else if let Ok((range, val)) = If::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::If(Box::new(val)));
            } else if let Ok((range, val)) = Compare::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Compare(Box::new(val)));
            } else if let Ok((range, _)) = convert.meta_bool("try") {
                convert.update(range);
                result = Some(Expression::Try(Box::new(result.unwrap())));
            } else if let Ok((range, val)) = Swizzle::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Swizzle(Box::new(val)));
            } else if let Ok((range, val)) = Closure::from_meta_data(
                    file, source, "closure", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Closure(Arc::new(val)));
            } else if let Ok((range, val)) = CallClosure::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::CallClosure(Box::new(val)));
            } else if let Ok((range, val)) = CallClosure::named_from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::CallClosure(Box::new(val)));
            } else if let Ok((range, val)) = Grab::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                if let Some(v) = val.precompute() {
                    result = Some(Expression::Variable(val.source_range, v));
                } else {
                    result = Some(Expression::Grab(Box::new(val)));
                }
            } else if let Ok((range, val)) = TryExpr::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::TryExpr(Box::new(val)));
            } else if let Ok((range, val)) = In::from_meta_data(
                    "in", convert, ignored) {
                convert.update(range);
                result = Some(Expression::In(Box::new(val)));
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let result = try!(result.ok_or(()));
        Ok((convert.subtract(start), result))
    }

    fn precompute(&self) -> Option<Variable> {
        use self::Expression::*;

        match *self {
            ArrayFill(ref array_fill) => array_fill.precompute(),
            Array(ref array) => array.precompute(),
            Variable(_, ref v) => Some(v.clone()),
            _ => None
        }
    }

    pub fn source_range(&self) -> Range {
        use self::Expression::*;

        match *self {
            Link(ref link) => link.source_range,
            Object(ref obj) => obj.source_range,
            Array(ref arr) => arr.source_range,
            ArrayFill(ref arr_fill) => arr_fill.source_range,
            Return(ref expr) => expr.source_range(),
            ReturnVoid(range) => range,
            Break(ref br) => br.source_range,
            Continue(ref c) => c.source_range,
            Block(ref bl) => bl.source_range,
            Go(ref go) => go.source_range,
            Call(ref call) => call.source_range,
            Item(ref it) => it.source_range,
            BinOp(ref binop) => binop.source_range,
            Assign(ref assign) => assign.source_range,
            Vec4(ref vec4) => vec4.source_range,
            For(ref for_expr) => for_expr.source_range,
            ForN(ref for_n_expr) => for_n_expr.source_range,
            Sum(ref for_n_expr) => for_n_expr.source_range,
            SumVec4(ref for_n_expr) => for_n_expr.source_range,
            Prod(ref for_n_expr) => for_n_expr.source_range,
            ProdVec4(ref for_n_expr) => for_n_expr.source_range,
            Min(ref for_n_expr) => for_n_expr.source_range,
            Max(ref for_n_expr) => for_n_expr.source_range,
            Sift(ref for_n_expr) => for_n_expr.source_range,
            Any(ref for_n_expr) => for_n_expr.source_range,
            All(ref for_n_expr) => for_n_expr.source_range,
            LinkFor(ref for_n_expr) => for_n_expr.source_range,
            If(ref if_expr) => if_expr.source_range,
            Compare(ref comp) => comp.source_range,
            Norm(ref norm) => norm.source_range,
            UnOp(ref unop) => unop.source_range,
            Variable(range, _) => range,
            Try(ref expr) => expr.source_range(),
            Swizzle(ref swizzle) => swizzle.source_range,
            Closure(ref closure) => closure.source_range,
            CallClosure(ref call) => call.source_range,
            Grab(ref grab) => grab.source_range,
            TryExpr(ref try_expr) => try_expr.source_range,
            In(ref in_expr) => in_expr.source_range,
        }
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        use self::Expression::*;

        match *self {
            Link(ref link) =>
                link.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Object(ref obj) =>
                obj.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Array(ref arr) =>
                arr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            ArrayFill(ref arr_fill) =>
                arr_fill.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Return(ref expr) => {
                let st = stack.len();
                expr.resolve_locals(relative, stack, closure_stack, module, use_lookup);
                stack.truncate(st);
            }
            ReturnVoid(_) => {}
            Break(_) => {}
            Continue(_) => {}
            Block(ref bl) =>
                bl.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Go(ref go) =>
                go.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Call(ref call) =>
                call.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Item(ref it) =>
                it.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            BinOp(ref binop) =>
                binop.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Assign(ref assign) =>
                assign.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Vec4(ref vec4) =>
                vec4.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            For(ref for_expr) =>
                for_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            ForN(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Sum(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            SumVec4(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Prod(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            ProdVec4(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Min(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Max(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Sift(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Any(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            All(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            LinkFor(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            If(ref if_expr) =>
                if_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Compare(ref comp) =>
                comp.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Norm(ref norm) =>
                norm.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            UnOp(ref unop) =>
                unop.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Variable(_, _) => {}
            Try(ref expr) =>
                expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Swizzle(ref swizzle) =>
                swizzle.expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Closure(ref closure) =>
                closure.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            CallClosure(ref call) =>
                call.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Grab(ref grab) =>
                grab.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            TryExpr(ref try_expr) =>
                try_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            In(ref in_expr) =>
                in_expr.resolve_locals(relative, module, use_lookup),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Link {
    pub items: Vec<Expression>,
    pub source_range: Range,
}

impl Link {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Link), ()> {
        let start = convert.clone();
        let node = "link";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut items: Vec<Expression> = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "link_item", convert, ignored) {
                convert.update(range);
                items.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), Link {
            items: items,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for expr in &self.items {
            expr.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        }
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    pub key_values: Vec<(Arc<String>, Expression)>,
    pub source_range: Range,
}

impl Object {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Object), ()> {
        let start = convert.clone();
        let node = "object";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut key_values = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Object::key_value_from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                key_values.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), Object {
            key_values: key_values,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn key_value_from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, (Arc<String>, Expression)), ()> {
        let start = convert.clone();
        let node = "key_value";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut key: Option<Arc<String>> = None;
        let mut value: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("key") {
                convert.update(range);
                key = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "val", convert, ignored) {
                convert.update(range);
                value = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let key = try!(key.ok_or(()));
        let value = try!(value.ok_or(()));
        Ok((convert.subtract(start), (key, value)))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for &(_, ref expr) in &self.key_values {
            expr.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(st);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    pub items: Vec<Expression>,
    pub source_range: Range,
}

impl Array {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Array), ()> {
        let start = convert.clone();
        let node = "array";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut items = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "array_item", convert, ignored) {
                convert.update(range);
                items.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), Array {
            items: items,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn precompute(&self) -> Option<Variable> {
        let mut res = Vec::with_capacity(self.items.len());
        for item in &self.items {
            if let Some(v) = item.precompute() {
                res.push(v);
            } else {
                return None;
            }
        }
        Some(Variable::Array(Arc::new(res)))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for item in &self.items {
            item.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(st);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArrayFill {
    pub fill: Expression,
    pub n: Expression,
    pub source_range: Range,
}

impl ArrayFill {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, ArrayFill), ()> {
        let start = convert.clone();
        let node = "array_fill";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut fill: Option<Expression> = None;
        let mut n: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "fill", convert, ignored) {
                convert.update(range);
                fill = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "n", convert, ignored) {
                convert.update(range);
                n = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let fill = try!(fill.ok_or(()));
        let n = try!(n.ok_or(()));
        Ok((convert.subtract(start), ArrayFill {
            fill: fill,
            n: n,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn precompute(&self) -> Option<Variable> {
        if let Expression::Variable(_, Variable::F64(n, _)) = self.n {
            if let Expression::Variable(_, ref x) = self.fill {
                return Some(Variable::Array(Arc::new(vec![x.clone(); n as usize])));
            }
        }
        None
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.fill.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
        self.n.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Add {
    pub items: Vec<Expression>,
    pub ops: Vec<BinOp>,
    pub source_range: Range,
}

impl Add {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Add), ()> {
        let start = convert.clone();
        let node = "add";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut items = vec![];
        let mut ops = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "expr", convert, ignored) {
                convert.update(range);
                items.push(val);
            } else if let Ok((range, _)) = convert.meta_bool("+") {
                convert.update(range);
                ops.push(BinOp::Add);
            } else if let Ok((range, _)) = convert.meta_bool("-") {
                convert.update(range);
                ops.push(BinOp::Sub);
            } else if let Ok((range, _)) = convert.meta_bool("||") {
                convert.update(range);
                ops.push(BinOp::OrElse);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        if items.len() == 0 {
            return Err(())
        }
        Ok((convert.subtract(start), Add {
            items: items,
            ops: ops,
            source_range: convert.source(start).unwrap()
        }))
    }

    pub fn to_expression(mut self) -> Expression {
        if self.items.len() == 1 {
            self.items[0].clone()
        } else {
            let op = self.ops.pop().unwrap();
            let last = self.items.pop().unwrap();
            let source_range = self.source_range;
            Expression::BinOp(Box::new(BinOpExpression {
                op: op,
                left: self.to_expression(),
                right: last,
                source_range: source_range
            }))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mul {
    pub items: Vec<Expression>,
    pub ops: Vec<BinOp>,
    pub source_range: Range,
}

impl Mul {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Mul), ()> {
        let start = convert.clone();
        let node = "mul";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut items = vec![];
        let mut ops = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = UnOpExpression::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                items.push(Expression::UnOp(Box::new(val)));
            } else if let Ok((range, val)) = Pow::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                items.push(val.to_expression());
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "val", convert, ignored) {
                convert.update(range);
                items.push(val);
            } else if let Ok((range, _)) = convert.meta_bool("*.") {
                convert.update(range);
                ops.push(BinOp::Dot);
            } else if let Ok((range, _)) = convert.meta_bool("x") {
                convert.update(range);
                ops.push(BinOp::Cross);
            } else if let Ok((range, _)) = convert.meta_bool("*") {
                convert.update(range);
                ops.push(BinOp::Mul);
            } else if let Ok((range, _)) = convert.meta_bool("/") {
                convert.update(range);
                ops.push(BinOp::Div);
            } else if let Ok((range, _)) = convert.meta_bool("%") {
                convert.update(range);
                ops.push(BinOp::Rem);
            } else if let Ok((range, _)) = convert.meta_bool("&&") {
                convert.update(range);
                ops.push(BinOp::AndAlso);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        if items.len() == 0 {
            return Err(())
        }
        Ok((convert.subtract(start), Mul {
            items: items,
            ops: ops,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn to_expression(mut self) -> Expression {
        if self.items.len() == 1 {
            self.items[0].clone()
        } else {
            let op = self.ops.pop().expect("Expected a binary operation");
            let last = self.items.pop().expect("Expected argument");
            let source_range = self.source_range;
            Expression::BinOp(Box::new(BinOpExpression {
                op: op,
                left: self.to_expression(),
                right: last,
                source_range: source_range,
            }))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pow {
    pub base: Expression,
    pub exp: Expression,
    pub source_range: Range,
}

impl Pow {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Pow), ()> {
        let start = convert.clone();
        let node = "pow";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut base: Option<Expression> = None;
        let mut exp: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "base", convert, ignored) {
                convert.update(range);
                base = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "exp", convert, ignored) {
                convert.update(range);
                exp = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let base = try!(base.ok_or(()));
        let exp = try!(exp.ok_or(()));
        Ok((convert.subtract(start), Pow {
            base: base,
            exp: exp,
            source_range: convert.source(start).unwrap()
        }))
    }

    pub fn to_expression(self) -> Expression {
        Expression::BinOp(Box::new(BinOpExpression {
                op: BinOp::Pow,
                left: self.base,
                right: self.exp,
                source_range: self.source_range,
        }))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Dot,
    Cross,
    Div,
    Rem,
    Pow,
    OrElse,
    AndAlso,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Dot => "*.",
            BinOp::Cross => "x",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Pow => "^",
            BinOp::OrElse => "||",
            BinOp::AndAlso => "&&",
        }
    }

    pub fn symbol_bool(self) -> &'static str {
        match self {
            BinOp::Add => "or",
            BinOp::Mul => "and",
            _ => self.symbol()
        }
    }

    /// Returns the operator precedence level.
    pub fn precedence(self) -> u8 {
        match self {
            BinOp::OrElse | BinOp::AndAlso => 0,
            BinOp::Add | BinOp::Sub => 1,
            BinOp::Mul | BinOp::Dot | BinOp::Cross
            | BinOp::Div | BinOp::Rem => 2,
            BinOp::Pow => 3,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum UnOp {
    Not,
    Neg,
}

#[derive(Debug, Clone)]
pub enum Id {
    String(Range, Arc<String>),
    F64(Range, f64),
    Expression(Expression),
}

impl Id {
    pub fn source_range(&self) -> Range {
        match *self {
            Id::String(range, _) => range,
            Id::F64(range, _) => range,
            Id::Expression(ref expr) => expr.source_range(),
        }
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) -> bool {
        match *self {
            Id::String(_, _) => false,
            Id::F64(_, _) => false,
            Id::Expression(ref expr) => {
                let st = stack.len();
                expr.resolve_locals(relative, stack, closure_stack, module, use_lookup);
                stack.truncate(st);
                true
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: Arc<String>,
    pub stack_id: Cell<Option<usize>>,
    pub static_stack_id: Cell<Option<usize>>,
    pub current: bool,
    pub try: bool,
    pub ids: Vec<Id>,
    // Stores indices of ids that should propagate errors.
    pub try_ids: Vec<usize>,
    pub source_range: Range,
}

impl Item {
    pub fn from_variable(name: Arc<String>, source_range: Range) -> Item {
        Item {
            name: name,
            current: false,
            stack_id: Cell::new(None),
            static_stack_id: Cell::new(None),
            try: false,
            ids: vec![],
            try_ids: vec![],
            source_range: source_range
        }
    }

    /// Truncates item extra to a given length.
    pub fn trunc(&self, n: usize) -> Item {
        Item {
            name: self.name.clone(),
            current: self.current,
            stack_id: Cell::new(None),
            static_stack_id: Cell::new(None),
            try: self.try,
            ids: self.ids.iter().take(n).map(|id| id.clone()).collect(),
            try_ids: {
                let mut try_ids = vec![];
                for &ind in &self.try_ids {
                    if ind >= n { break }
                    try_ids.push(ind);
                }
                try_ids
            },
            source_range: self.source_range
        }
    }

    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Item), ()> {
        let start = convert.clone();
        let node = "item";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
        let mut current = false;
        let mut ids = vec![];
        let mut try_ids = vec![];
        let mut try = false;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else if let Ok((range, _)) = convert.meta_bool("current") {
                convert.update(range);
                current = true;
            } else if let Ok((range, _)) = convert.meta_bool("try_item") {
                convert.update(range);
                try = true;
                // Ignore item extra node, which is there to help the type checker.
            } else if let Ok(range) = convert.start_node("item_extra") {
                convert.update(range);
            } else if let Ok(range) = convert.end_node("item_extra") {
                convert.update(range);
            } else if let Ok((range, val)) = convert.meta_string("id") {
                let start_id = convert;
                convert.update(range);
                ids.push(Id::String(convert.source(start_id).unwrap(), val));
            } else if let Ok((range, val)) = convert.meta_f64("id") {
                let start_id = convert;
                convert.update(range);
                ids.push(Id::F64(convert.source(start_id).unwrap(), val));
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "id", convert, ignored) {
                convert.update(range);
                ids.push(Id::Expression(val));
            } else if let Ok((range, _)) = convert.meta_bool("try_id") {
                convert.update(range);
                // id is pushed before the `?` operator, therefore subtract 1.
                try_ids.push(ids.len() - 1);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let name = try!(name.ok_or(()));
        Ok((convert.subtract(start), Item {
            name: name,
            stack_id: Cell::new(None),
            static_stack_id: Cell::new(None),
            current: current,
            try: try,
            ids: ids,
            try_ids: try_ids,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        // println!("TEST item resolve {} {:?}", self.name, stack);
        let st = stack.len();
        for (i, n) in stack.iter().rev().enumerate() {
            if let &Some(ref n) = n {
                if &**n == &**self.name {
                    // println!("TEST set {} {}", self.name, i + 1);
                    self.static_stack_id.set(Some(i + 1));
                    break;
                }
            }
        }
        for id in &self.ids {
            if id.resolve_locals(relative, stack, closure_stack, module, use_lookup) {
                stack.push(None);
            }
        }
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Go {
    pub call: Call,
    pub source_range: Range,
}

impl Go {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Go), ()> {
        let start = convert.clone();
        let node = "go";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut call: Option<Call> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Call::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                call = Some(val);
            } else if let Ok((range, val)) = Call::named_from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                call = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let call = try!(call.ok_or(()));
        Ok((convert.subtract(start), Go {
            call: call,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for arg in &self.call.args {
            let st = stack.len();
            arg.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(st);
        }
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub alias: Option<Arc<String>>,
    pub name: Arc<String>,
    pub args: Vec<Expression>,
    pub f_index: Cell<FnIndex>,
    /// A custom source, such as when calling a function inside a loaded module.
    pub custom_source: Option<Arc<String>>,
    pub source_range: Range,
}

impl Call {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Call), ()> {
        let start = convert.clone();
        let node = "call";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut alias: Option<Arc<String>> = None;
        let mut name: Option<Arc<String>> = None;
        let mut args = vec![];
        let mut mutable: Vec<bool> = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("alias") {
                convert.update(range);
                alias = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "call_arg", convert, ignored) {
                let mut peek = convert.clone();
                mutable.push(match peek.start_node("call_arg") {
                    Ok(r) => {
                        peek.update(r);
                        peek.meta_bool("mut").is_ok()
                    }
                    _ => unreachable!()
                });
                convert.update(range);
                args.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let mut name = try!(name.ok_or(()));

        // Append mutability information to function name.
        if mutable.iter().any(|&arg| arg) {
            let mut name_plus_args = String::from(&**name);
            name_plus_args.push('(');
            let mut first = true;
            for &arg in &mutable {
                if !first { name_plus_args.push(','); }
                name_plus_args.push_str(if arg { "mut" } else { "_" });
                first = false;
            }
            name_plus_args.push(')');
            name = Arc::new(name_plus_args);
        }

        Ok((convert.subtract(start), Call {
            alias: alias,
            name: name,
            args: args,
            f_index: Cell::new(FnIndex::None),
            custom_source: None,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn named_from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Call), ()> {
        let start = convert.clone();
        let node = "named_call";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut alias: Option<Arc<String>> = None;
        let mut name = String::new();
        let mut args = vec![];
        let mut mutable: Vec<bool> = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("alias") {
                convert.update(range);
                alias = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("word") {
                convert.update(range);
                if name.len() != 0 {
                    name.push('_');
                    name.push_str(&val);
                } else {
                    name.push_str(&val);
                    name.push('_');
                }
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "call_arg", convert, ignored) {
                let mut peek = convert.clone();
                mutable.push(match peek.start_node("call_arg") {
                    Ok(r) => {
                        peek.update(r);
                        peek.meta_bool("mut").is_ok()
                    }
                    _ => unreachable!()
                });
                convert.update(range);
                args.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        // Append mutability information to function name.
        if mutable.iter().any(|&arg| arg) {
            name.push('(');
            let mut first = true;
            for &arg in &mutable {
                if !first { name.push(','); }
                name.push_str(if arg { "mut" } else { "_" });
                first = false;
            }
            name.push(')');
        }

        Ok((convert.subtract(start), Call {
            alias: alias,
            name: Arc::new(name),
            args: args,
            f_index: Cell::new(FnIndex::None),
            custom_source: None,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        let f_index = if let Some(ref alias) = self.alias {
            if let Some(&i) = use_lookup.aliases.get(alias).and_then(|map| map.get(&self.name)) {
                FnIndex::Loaded(i as isize - relative as isize)
            } else {
                FnIndex::None
            }
        } else {
            module.find_function(&self.name, relative)
        };
        self.f_index.set(f_index);
        match f_index {
            FnIndex::Loaded(f_index) => {
                let index = (f_index + relative as isize) as usize;
                if module.functions[index].returns() {
                    stack.push(None);
                }
            }
            FnIndex::ExternalVoid(_) | FnIndex::ExternalReturn(_) => {
                // Don't push return since last value in block
                // is used as return value.
            }
            FnIndex::Intrinsic(_) => {}
            FnIndex::None => {}
        }
        for arg in &self.args {
            let arg_st = stack.len();
            arg.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(arg_st);
            match *arg {
                Expression::Swizzle(ref swizzle) => {
                    for _ in 0..swizzle.len() {
                        stack.push(None);
                    }
                }
                _ => {
                    stack.push(None);
                }
            }
        }
        stack.truncate(st);
    }

    /// Computes number of arguments including swizzles.
    pub fn arg_len(&self) -> usize {
        let mut sum = 0;
        for arg in &self.args {
            match *arg {
                Expression::Swizzle(ref swizzle) => {
                    sum += swizzle.len();
                }
                _ => { sum += 1; }
            }
        }
        sum
    }
}

#[derive(Debug, Clone)]
pub struct CallClosure {
    pub item: Item,
    pub args: Vec<Expression>,
    pub source_range: Range,
}

impl CallClosure {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, CallClosure), ()> {
        let start = convert.clone();
        let node = "call_closure";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut item: Option<Item> = None;
        let mut args = vec![];
        let mut mutable: Vec<bool> = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Item::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                item = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "call_arg", convert, ignored) {
                let mut peek = convert.clone();
                mutable.push(match peek.start_node("call_arg") {
                    Ok(r) => {
                        peek.update(r);
                        peek.meta_bool("mut").is_ok()
                    }
                    _ => unreachable!()
                });
                convert.update(range);
                args.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let item = try!(item.ok_or(()));
        Ok((convert.subtract(start), CallClosure {
            item: item,
            args: args,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn named_from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, CallClosure), ()> {
        let start = convert.clone();
        let node = "named_call_closure";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut item: Option<Item> = None;
        let mut name = String::new();
        let mut args = vec![];
        let mut mutable: Vec<bool> = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Item::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                item = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("word") {
                convert.update(range);
                if name.len() != 0 {
                    name.push('_');
                    name.push_str(&val);
                } else {
                    name.push_str(&val);
                }
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "call_arg", convert, ignored) {
                let mut peek = convert.clone();
                mutable.push(match peek.start_node("call_arg") {
                    Ok(r) => {
                        peek.update(r);
                        peek.meta_bool("mut").is_ok()
                    }
                    _ => unreachable!()
                });
                convert.update(range);
                args.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let mut item = try!(item.ok_or(()));
        {
            if item.ids.len() == 0 {
                // Append name to item.
                let n = Arc::make_mut(&mut item.name);
                n.push_str("__");
                n.push_str(&name);
            } else {
                let last = item.ids.len() - 1;
                if let Id::String(_, ref mut n) = item.ids[last] {
                    // Append name to last id.
                    let n = Arc::make_mut(n);
                    n.push_str("__");
                    n.push_str(&name);
                }
            }
        }
        Ok((convert.subtract(start), CallClosure {
            item: item,
            args: args,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.item.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        // All closures must return a value.
        stack.push(Some(Arc::new("return".into())));
        for arg in &self.args {
            let arg_st = stack.len();
            arg.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(arg_st);
            match *arg {
                Expression::Swizzle(ref swizzle) => {
                    for _ in 0..swizzle.len() {
                        stack.push(None);
                    }
                }
                _ => {
                    stack.push(None);
                }
            }
        }
        stack.truncate(st);
    }

    /// Computes number of arguments including swizzles.
    pub fn arg_len(&self) -> usize {
        let mut sum = 0;
        for arg in &self.args {
            match *arg {
                Expression::Swizzle(ref swizzle) => {
                    sum += swizzle.len();
                }
                _ => { sum += 1; }
            }
        }
        sum
    }
}

#[derive(Debug, Clone)]
pub struct Norm {
    pub expr: Expression,
    pub source_range: Range,
}

impl Norm {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Norm), ()> {
        let start = convert.clone();
        let node = "norm";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut expr: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "expr", convert, ignored) {
                convert.update(range);
                expr = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let expr = try!(expr.ok_or(()));
        Ok((convert.subtract(start), Norm {
            expr: expr,
            source_range: convert.source(start).unwrap()
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.expr.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct BinOpExpression {
    pub op: BinOp,
    pub left: Expression,
    pub right: Expression,
    pub source_range: Range,
}

impl BinOpExpression {
    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.left.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
        self.right.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct UnOpExpression {
    pub op: UnOp,
    pub expr: Expression,
    pub source_range: Range,
}

impl UnOpExpression {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, UnOpExpression), ()> {
        let start = convert.clone();
        let node = "unop";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut unop: Option<UnOp> = None;
        let mut expr: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, _)) = convert.meta_bool("!") {
                convert.update(range);
                unop = Some(UnOp::Not);
            } else if let Ok((range, _)) = convert.meta_bool("-") {
                convert.update(range);
                unop = Some(UnOp::Neg);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "expr", convert, ignored) {
                convert.update(range);
                expr = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let unop = try!(unop.ok_or(()));
        let expr = try!(expr.ok_or(()));
        Ok((convert.subtract(start), UnOpExpression {
            op: unop,
            expr: expr,
            source_range: convert.source(start).unwrap()
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.expr.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub op: AssignOp,
    pub left: Expression,
    pub right: Expression,
    pub source_range: Range,
}

impl Assign {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Assign), ()> {
        let start = convert.clone();
        let node = "assign";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut op: Option<AssignOp> = None;
        let mut left: Option<Expression> = None;
        let mut right: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, _)) = convert.meta_bool(":=") {
                convert.update(range);
                op = Some(AssignOp::Assign);
            } else if let Ok((range, _)) = convert.meta_bool("=") {
                convert.update(range);
                op = Some(AssignOp::Set);
            } else if let Ok((range, _)) = convert.meta_bool("+=") {
                convert.update(range);
                op = Some(AssignOp::Add);
            } else if let Ok((range, _)) = convert.meta_bool("-=") {
                convert.update(range);
                op = Some(AssignOp::Sub);
            } else if let Ok((range, _)) = convert.meta_bool("*=") {
                convert.update(range);
                op = Some(AssignOp::Mul);
            } else if let Ok((range, _)) = convert.meta_bool("/=") {
                convert.update(range);
                op = Some(AssignOp::Div);
            } else if let Ok((range, _)) = convert.meta_bool("%=") {
                convert.update(range);
                op = Some(AssignOp::Rem);
            } else if let Ok((range, _)) = convert.meta_bool("^=") {
                convert.update(range);
                op = Some(AssignOp::Pow);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "left", convert, ignored) {
                convert.update(range);
                left = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "right", convert, ignored) {
                convert.update(range);
                right = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let op = try!(op.ok_or(()));
        let left = try!(left.ok_or(()));
        let right = try!(right.ok_or(()));
        Ok((convert.subtract(start), Assign {
            op: op,
            left: left,
            right: right,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        // Declared locals in right expressions are popped from the stack.
        let st = stack.len();
        self.right.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);

        // Declare new local when there is an item with no extra.
        if let Expression::Item(ref item) = self.left {
            if item.ids.len() == 0 && self.op == AssignOp::Assign {
                stack.push(Some(item.name.clone()));
                return;
            }
        }

        self.left.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AssignOp {
    /// :=
    Assign,
    /// =
    Set,
    /// +=
    Add,
    /// -=
    Sub,
    /// *=
    Mul,
    /// /=
    Div,
    /// %=
    Rem,
    /// ^=
    Pow,
}

impl AssignOp {
    pub fn symbol(&self) -> &'static str {
        use self::AssignOp::*;

        match *self {
            Assign => ":=",
            Set => "=",
            Add => "+=",
            Sub => "-=",
            Mul => "*=",
            Div => "/=",
            Rem => "%=",
            Pow => "^=",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Vec4 {
    pub args: Vec<Expression>,
    pub source_range: Range,
}

impl Vec4 {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Vec4), ()> {
        let start = convert.clone();
        let node = "vec4";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut x: Option<Expression> = None;
        let mut y: Option<Expression> = None;
        let mut z: Option<Expression> = None;
        let mut w: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "x", convert, ignored) {
                convert.update(range);
                x = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "y", convert, ignored) {
                convert.update(range);
                y = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "z", convert, ignored) {
                convert.update(range);
                z = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "w", convert, ignored) {
                convert.update(range);
                w = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let x = try!(x.ok_or(()));
        let y = y.unwrap_or(Expression::Variable(Range::empty(0), Variable::f64(0.0)));
        let z = z.unwrap_or(Expression::Variable(Range::empty(0), Variable::f64(0.0)));
        let w = w.unwrap_or(Expression::Variable(Range::empty(0), Variable::f64(0.0)));
        Ok((convert.subtract(start), Vec4 {
            args: vec![x, y, z, w],
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for arg in &self.args {
            let arg_st = stack.len();
            arg.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(arg_st);
            match *arg {
                Expression::Swizzle(ref swizzle) => {
                    for _ in 0..swizzle.len() {
                        stack.push(None);
                    }
                }
                _ => {
                    stack.push(None);
                }
            }
        }
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Vec4UnLoop {
    pub name: Arc<String>,
    pub expr: Expression,
    pub len: u8,
    pub source_range: Range,
}

impl Vec4UnLoop {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Vec4UnLoop), ()> {
        let start = convert.clone();
        let node = "vec4_un_loop";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
        let mut expr: Option<Expression> = None;
        let mut len: u8 = 4;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, _)) = convert.meta_bool("4") {
                convert.update(range);
                len = 4;
            } else if let Ok((range, _)) = convert.meta_bool("3") {
                convert.update(range);
                len = 3;
            } else if let Ok((range, _)) = convert.meta_bool("2") {
                convert.update(range);
                len = 2;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "expr", convert, ignored) {
                convert.update(range);
                expr = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let name = try!(name.ok_or(()));
        let expr = try!(expr.ok_or(()));
        Ok((convert.subtract(start), Vec4UnLoop {
            name: name,
            expr: expr,
            len: len,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn to_expression(self) -> Expression {
        let source_range = self.source_range;

        let zero = || Expression::Variable(source_range, Variable::f64(0.0));

        let replace_0 = replace::number(&self.expr, &self.name, 0.0);
        let replace_1 = replace::number(&self.expr, &self.name, 1.0);
        let replace_2 = if self.len > 2 {
                replace::number(&self.expr, &self.name, 2.0)
            } else {
                zero()
            };
        let replace_3 = if self.len > 3 {
                replace::number(&self.expr, &self.name, 3.0)
            } else {
                zero()
            };

        Expression::Vec4(Vec4 {
            args: vec![replace_0, replace_1, replace_2, replace_3],
            source_range: source_range,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Swizzle {
    pub sw0: usize,
    pub sw1: usize,
    pub sw2: Option<usize>,
    pub sw3: Option<usize>,
    pub expr: Expression,
    pub source_range: Range,
}

impl Swizzle {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Swizzle), ()> {
        let start = convert.clone();
        let node = "swizzle";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut sw0: Option<usize> = None;
        let mut sw1: Option<usize> = None;
        let mut sw2: Option<usize> = None;
        let mut sw3: Option<usize> = None;
        let mut expr: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Sw::from_meta_data(
                "sw0", convert, ignored) {
                convert.update(range);
                sw0 = Some(val.ind);
            } else if let Ok((range, val)) = Sw::from_meta_data(
                "sw1", convert, ignored) {
                convert.update(range);
                sw1 = Some(val.ind);
            } else if let Ok((range, val)) = Sw::from_meta_data(
                "sw2", convert, ignored) {
                convert.update(range);
                sw2 = Some(val.ind);
            } else if let Ok((range, val)) = Sw::from_meta_data(
                "sw3", convert, ignored) {
                convert.update(range);
                sw3 = Some(val.ind);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "expr", convert, ignored) {
                convert.update(range);
                expr = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let sw0 = try!(sw0.ok_or(()));
        let sw1 = try!(sw1.ok_or(()));
        let expr = try!(expr.ok_or(()));
        Ok((convert.subtract(start), Swizzle {
            sw0: sw0,
            sw1: sw1,
            sw2: sw2,
            sw3: sw3,
            expr: expr,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn len(&self) -> usize {
        return 2 +
            if self.sw2.is_some() { 1 } else { 0 } +
            if self.sw3.is_some() { 1 } else { 0 }
    }
}

#[derive(Debug, Clone)]
pub struct Sw {
    pub ind: usize,
    pub source_range: Range,
}

impl Sw {
    pub fn from_meta_data(
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Sw), ()> {
        let start = convert.clone();
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut ind: Option<usize> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, _)) = convert.meta_bool("x") {
                convert.update(range);
                ind = Some(0);
            } else if let Ok((range, _)) = convert.meta_bool("y") {
                convert.update(range);
                ind = Some(1);
            } else if let Ok((range, _)) = convert.meta_bool("z") {
                convert.update(range);
                ind = Some(2);
            } else if let Ok((range, _)) = convert.meta_bool("w") {
                convert.update(range);
                ind = Some(3);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let ind = try!(ind.ok_or(()));
        Ok((convert.subtract(start), Sw {
            ind: ind,
            source_range: convert.source(start).unwrap(),
        }))
    }
}

#[derive(Debug, Clone)]
pub struct For {
    pub init: Expression,
    pub cond: Expression,
    pub step: Expression,
    pub block: Block,
    pub label: Option<Arc<String>>,
    pub source_range: Range,
}

impl For {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, For), ()> {
        let start = convert.clone();
        let node = "for";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut init: Option<Expression> = None;
        let mut cond: Option<Expression> = None;
        let mut step: Option<Expression> = None;
        let mut block: Option<Block> = None;
        let mut label: Option<Arc<String>> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "init", convert, ignored) {
                convert.update(range);
                init = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "cond", convert, ignored) {
                convert.update(range);
                cond = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "step", convert, ignored) {
                convert.update(range);
                step = Some(val);
            } else if let Ok((range, val)) = Block::from_meta_data(
                    file, source, "block", convert, ignored) {
                convert.update(range);
                block = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("label") {
                convert.update(range);
                label = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let init = try!(init.ok_or(()));
        let cond = try!(cond.ok_or(()));
        let step = try!(step.ok_or(()));
        let block = try!(block.ok_or(()));
        Ok((convert.subtract(start), For {
            init: init,
            cond: cond,
            step: step,
            block: block,
            label: label,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(
        &self, relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.init.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        let after_init = stack.len();
        self.cond.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(after_init);
        self.step.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(after_init);
        self.block.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct ForN {
    pub name: Arc<String>,
    pub start: Option<Expression>,
    pub end: Expression,
    pub block: Block,
    pub label: Option<Arc<String>>,
    pub source_range: Range,
}

impl ForN {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, ForN), ()> {
        let start = convert.clone();
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut indices: Vec<(Arc<String>, Option<Expression>, Option<Expression>)> = vec![];
        let mut block: Option<Block> = None;
        let mut label: Option<Arc<String>> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Block::from_meta_data(
                    file, source, "block", convert, ignored) {
                convert.update(range);
                block = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("label") {
                convert.update(range);
                label = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                let mut start_expr: Option<Expression> = None;
                let mut end_expr: Option<Expression> = None;
                if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "start", convert, ignored) {
                    convert.update(range);
                    start_expr = Some(val);
                }
                if let Ok((range, val)) = Expression::from_meta_data(
                    file, source, "end", convert, ignored) {
                    convert.update(range);
                    end_expr = Some(val);
                }
                indices.push((val, start_expr, end_expr));
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        ForN::create(
            node,
            convert.subtract(start),
            convert.source(start).unwrap(),
            label,
            &indices,
            block
        )
    }

    fn create(
        node: &str,
        range: Range,
        source_range: Range,
        label: Option<Arc<String>>,
        indices: &[(Arc<String>, Option<Expression>, Option<Expression>)],
        mut block: Option<Block>
    ) -> Result<(Range, ForN), ()> {
        if indices.len() == 0 { return Err(()); }

        let name: Arc<String> = indices[0].0.clone();
        let start_expr = indices[0].1.clone();
        let mut end_expr = indices[0].2.clone();

        if indices.len() > 1 {
            let (_, new_for_n) = try!(ForN::create(
                node,
                range,
                source_range,
                None,
                &indices[1..],
                block
            ));
            block = Some(Block {
                source_range: source_range,
                expressions: vec![match node {
                    "for_n" => Expression::ForN(Box::new(new_for_n)),
                    "sum" => Expression::Sum(Box::new(new_for_n)),
                    "sum_vec4" => Expression::SumVec4(Box::new(new_for_n)),
                    "prod" => Expression::Prod(Box::new(new_for_n)),
                    "prod_vec4" => Expression::ProdVec4(Box::new(new_for_n)),
                    "any" => Expression::Any(Box::new(new_for_n)),
                    "all" => Expression::All(Box::new(new_for_n)),
                    "min" => Expression::Min(Box::new(new_for_n)),
                    "max" => Expression::Max(Box::new(new_for_n)),
                    "sift" => Expression::Sift(Box::new(new_for_n)),
                    "link_for" => Expression::LinkFor(Box::new(new_for_n)),
                    _ => return Err(())
                }]
            });
        }

        let block = try!(block.ok_or(()));

        // Infer list length from index.
        if end_expr.is_none() {
            end_expr = infer_len::infer(&block, &name);
        }

        let end_expr = try!(end_expr.ok_or(()));
        Ok((range, ForN {
            name: name,
            start: start_expr,
            end: end_expr,
            block: block,
            label: label,
            source_range: source_range,
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        if let Some(ref start) = self.start {
            start.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(st);
        }
        self.end.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
        stack.push(Some(self.name.clone()));
        self.block.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub block: Block,
    pub label: Option<Arc<String>>,
    pub source_range: Range,
}

impl Loop {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Loop), ()> {
        let start = convert.clone();
        let node = "loop";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut block: Option<Block> = None;
        let mut label: Option<Arc<String>> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Block::from_meta_data(
                    file, source, "block", convert, ignored) {
                convert.update(range);
                block = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("label") {
                convert.update(range);
                label = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let block = try!(block.ok_or(()));
        Ok((convert.subtract(start), Loop {
            block: block,
            label: label,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn to_expression(self) -> Expression {
        let source_range = self.source_range;
        Expression::For(Box::new(For {
            block: self.block,
            label: self.label,
            init: Expression::Block(Block {
                expressions: vec![],
                source_range: source_range,
            }),
            step: Expression::Block(Block {
                expressions: vec![],
                source_range: source_range,
            }),
            cond: Expression::Variable(source_range, Variable::bool(true)),
            source_range: source_range,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Break {
    pub label: Option<Arc<String>>,
    pub source_range: Range,
}

impl Break {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Break), ()> {
        let start = convert.clone();
        let node = "break";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut label: Option<Arc<String>> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("label") {
                convert.update(range);
                label = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), Break {
            label: label,
            source_range: convert.source(start).unwrap(),
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Continue {
    pub label: Option<Arc<String>>,
    pub source_range: Range,
}

impl Continue {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Continue), ()> {
        let start = convert.clone();
        let node = "continue";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut label: Option<Arc<String>> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("label") {
                convert.update(range);
                label = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), Continue {
            label: label,
            source_range: convert.source(start).unwrap(),
        }))
    }
}

#[derive(Debug, Clone)]
pub struct If {
    pub cond: Expression,
    pub true_block: Block,
    pub else_if_conds: Vec<Expression>,
    pub else_if_blocks: Vec<Block>,
    pub else_block: Option<Block>,
    pub source_range: Range,
}

impl If {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, If), ()> {
        let start = convert.clone();
        let node = "if";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut cond: Option<Expression> = None;
        let mut true_block: Option<Block> = None;
        let mut else_if_conds: Vec<Expression> = vec![];
        let mut else_if_blocks: Vec<Block> = vec![];
        let mut else_block: Option<Block> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "cond", convert, ignored) {
                convert.update(range);
                cond = Some(val);
            } else if let Ok((range, val)) = Block::from_meta_data(
                    file, source, "true_block", convert, ignored) {
                convert.update(range);
                true_block = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "else_if_cond", convert, ignored) {
                convert.update(range);
                else_if_conds.push(val);
            } else if let Ok((range, val)) = Block::from_meta_data(
                    file, source, "else_if_block", convert, ignored) {
                convert.update(range);
                else_if_blocks.push(val);
            } else if let Ok((range, val)) = Block::from_meta_data(
                    file, source, "else_block", convert, ignored) {
                convert.update(range);
                else_block = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let cond = try!(cond.ok_or(()));
        let true_block = try!(true_block.ok_or(()));
        Ok((convert.subtract(start), If {
            cond: cond,
            true_block: true_block,
            else_if_conds: else_if_conds,
            else_if_blocks: else_if_blocks,
            else_block: else_block,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.cond.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
        self.true_block.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
        // Does not matter that conditions are resolved before blocks,
        // since the stack gets truncated anyway.
        for else_if_cond in &self.else_if_conds {
            else_if_cond.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(st);
        }
        for else_if_block in &self.else_if_blocks {
            else_if_block.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(st);
        }
        if let Some(ref else_block) = self.else_block {
            else_block.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(st);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Compare {
    pub op: CompareOp,
    pub left: Expression,
    pub right: Expression,
    pub source_range: Range,
}

impl Compare {
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Compare), ()> {
        let start = convert.clone();
        let node = "compare";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut op: Option<CompareOp> = None;
        let mut left: Option<Expression> = None;
        let mut right: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, _)) = convert.meta_bool("<") {
                convert.update(range);
                op = Some(CompareOp::Less);
            } else if let Ok((range, _)) = convert.meta_bool("<=") {
                convert.update(range);
                op = Some(CompareOp::LessOrEqual);
            } else if let Ok((range, _)) = convert.meta_bool(">") {
                convert.update(range);
                op = Some(CompareOp::Greater);
            } else if let Ok((range, _)) = convert.meta_bool(">=") {
                convert.update(range);
                op = Some(CompareOp::GreaterOrEqual);
            } else if let Ok((range, _)) = convert.meta_bool("==") {
                convert.update(range);
                op = Some(CompareOp::Equal);
            } else if let Ok((range, _)) = convert.meta_bool("!=") {
                convert.update(range);
                op = Some(CompareOp::NotEqual);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "left", convert, ignored) {
                convert.update(range);
                left = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "right", convert, ignored) {
                convert.update(range);
                right = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let op = try!(op.ok_or(()));
        let left = try!(left.ok_or(()));
        let right = try!(right.ok_or(()));
        Ok((convert.subtract(start), Compare {
            op: op,
            left: left,
            right: right,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.left.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
        self.right.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompareOp {
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
}

impl CompareOp {
    pub fn symbol(self) -> &'static str {
        match self {
            CompareOp::Less => "<",
            CompareOp::LessOrEqual => "<=",
            CompareOp::Greater => ">",
            CompareOp::GreaterOrEqual => ">=",
            CompareOp::Equal => "==",
            CompareOp::NotEqual => "!=",
        }
    }
}

#[derive(Debug, Clone)]
pub struct In {
    pub alias: Option<Arc<String>>,
    pub name: Arc<String>,
    pub f_index: Cell<FnIndex>,
    pub source_range: Range,
}

impl In {
    pub fn from_meta_data(
        node: &'static str,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, In), ()> {
        let start = convert.clone();
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
        let mut alias: Option<Arc<String>> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("alias") {
                convert.update(range);
                alias = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let name = try!(name.ok_or(()));
        Ok((convert.subtract(start), In {
            alias,
            name,
            f_index: Cell::new(FnIndex::None),
            source_range: convert.source(start).unwrap()
        }))
    }

    pub fn resolve_locals(
        &self,
        relative: usize,
        module: &Module,
        use_lookup: &UseLookup
    ) {
        let f_index = if let Some(ref alias) = self.alias {
            if let Some(&i) = use_lookup.aliases.get(alias).and_then(|map| map.get(&self.name)) {
                FnIndex::Loaded(i as isize - relative as isize)
            } else {
                FnIndex::None
            }
        } else {
            module.find_function(&self.name, relative)
        };
        self.f_index.set(f_index);
    }
}
