//! Dyon Abstract Syntax Tree (AST).

use std::collections::HashMap;
use std::sync::{self, Arc};
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

/// Convert meta data and load it into a module.
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

/// Function alias.
#[derive(Copy, Clone)]
pub enum FnAlias {
    /// An alias to a loaded function.
    Loaded(usize),
    /// An alias to an external function.
    External(usize),
}

/// Used to resolve calls to imported functions.
pub struct UseLookup {
    /// Stores namespace aliases.
    /// The first key is the alias to namespace.
    /// The second key is the alias to the function.
    pub aliases: HashMap<Arc<String>, HashMap<Arc<String>, FnAlias>>,
}

impl Default for UseLookup {
    fn default() -> UseLookup {UseLookup::new()}
}

impl UseLookup {
    /// Creates a new use lookup.
    pub fn new() -> UseLookup {
        UseLookup {
            aliases: HashMap::new(),
        }
    }

    /// This is called when constructing the AST.
    pub fn from_uses_module(uses: &Uses, module: &Module) -> UseLookup {
        let mut aliases = HashMap::new();
        // First, add all glob imports.
        for use_import in &uses.use_imports {
            if !use_import.fns.is_empty() {continue;}
            if !aliases.contains_key(&use_import.alias) {
                aliases.insert(use_import.alias.clone(), HashMap::new());
            }
            let fns = aliases.get_mut(&use_import.alias).unwrap();
            for (i, f) in module.functions.iter().enumerate().rev() {
                if *f.namespace == use_import.names {
                    fns.insert(f.name.clone(), FnAlias::Loaded(i));
                }
            }
            for (i, f) in module.ext_prelude.iter().enumerate().rev() {
                if *f.namespace == use_import.names {
                    fns.insert(f.name.clone(), FnAlias::External(i));
                }
            }
        }
        // Second, add specific functions, which shadows glob imports.
        for use_import in &uses.use_imports {
            if use_import.fns.is_empty() {continue;}
            if !aliases.contains_key(&use_import.alias) {
                aliases.insert(use_import.alias.clone(), HashMap::new());
            }
            let fns = aliases.get_mut(&use_import.alias).unwrap();
            for use_fn in &use_import.fns {
                for (i, f) in module.functions.iter().enumerate().rev() {
                    if *f.namespace != use_import.names {continue;}
                    if f.name == use_fn.0 {
                        fns.insert(use_fn.1.as_ref().unwrap_or(&use_fn.0).clone(),
                                   FnAlias::Loaded(i));
                    } else if f.name.len() > use_fn.0.len() &&
                              f.name.starts_with(&**use_fn.0) &&
                              f.name.as_bytes()[use_fn.0.len()] == b'(' {
                        // A function with mutable information.
                        let mut name: Arc<String> = use_fn.1.as_ref().unwrap_or(&use_fn.0).clone();
                        Arc::make_mut(&mut name).push_str(&f.name.as_str()[use_fn.0.len()..]);
                        fns.insert(name, FnAlias::Loaded(i));
                    }
                }
                for (i, f) in module.ext_prelude.iter().enumerate().rev() {
                    if *f.namespace != use_import.names {continue;}
                    if f.name == use_fn.0 {
                        fns.insert(use_fn.1.as_ref().unwrap_or(&use_fn.0).clone(),
                                   FnAlias::External(i));
                    } else if f.name.len() > use_fn.0.len() &&
                              f.name.starts_with(&**use_fn.0) &&
                              f.name.as_bytes()[use_fn.0.len()] == b'(' {
                        // A function with mutable information.
                        let mut name: Arc<String> = use_fn.1.as_ref().unwrap_or(&use_fn.0).clone();
                        Arc::make_mut(&mut name).push_str(&f.name.as_str()[use_fn.0.len()..]);
                        fns.insert(f.name.clone(), FnAlias::External(i));
                    }
                }
            }
        }
        UseLookup {aliases}
    }

    /// This is called from lifetime/type checker.
    /// Here, external functions are treated as loaded.
    pub fn from_uses_prelude(uses: &Uses, prelude: &Prelude) -> UseLookup {
        let mut aliases = HashMap::new();
        // First, add all glob imports.
        for use_import in &uses.use_imports {
            if !use_import.fns.is_empty() {continue;}
            if !aliases.contains_key(&use_import.alias) {
                aliases.insert(use_import.alias.clone(), HashMap::new());
            }
            let fns = aliases.get_mut(&use_import.alias).unwrap();
            for (i, f) in prelude.namespaces.iter().enumerate().rev() {
                if *f.0 == use_import.names {
                    fns.insert(f.1.clone(), FnAlias::Loaded(i));
                }
            }
        }
        // Second, add specific functions, which shadows glob imports.
        for use_import in &uses.use_imports {
            if use_import.fns.is_empty() {continue;}
            if !aliases.contains_key(&use_import.alias) {
                aliases.insert(use_import.alias.clone(), HashMap::new());
            }
            let fns = aliases.get_mut(&use_import.alias).unwrap();
            for use_fn in &use_import.fns {
                for (i, f) in prelude.namespaces.iter().enumerate().rev() {
                    if *f.0 != use_import.names {continue;}
                    if f.1 == use_fn.0 {
                        fns.insert(use_fn.1.as_ref().unwrap_or(&use_fn.0).clone(),
                                   FnAlias::Loaded(i));
                    } else if f.1.len() > use_fn.0.len() &&
                              f.1.starts_with(&**use_fn.0) &&
                              f.1.as_bytes()[use_fn.0.len()] == b'(' {
                        // A function with mutable information.
                        let mut name: Arc<String> = use_fn.1.as_ref().unwrap_or(&use_fn.0).clone();
                        Arc::make_mut(&mut name).push_str(&f.1.as_str()[use_fn.0.len()..]);
                        fns.insert(name, FnAlias::Loaded(i));
                    }
                }
            }
        }
        UseLookup {aliases}
    }
}

/// Namespace, used to organize code in larger projects.
///
/// E.g. `ns math::algebra`.
#[derive(Debug, Clone)]
pub struct Namespace {
    /// Names separated by `::`.
    pub names: Arc<Vec<Arc<String>>>,
}

impl Namespace {
    /// Creates namespace from meta data.
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Namespace), ()> {
        let start = convert;
        let node = "ns";
        let start_range = convert.start_node(node)?;
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

/// Uses, lists use imports.
#[derive(Debug, Clone)]
pub struct Uses {
    /// List of use imports.
    pub use_imports: Vec<UseImport>,
}

impl Uses {
    /// Creates uses from meta data.
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Uses), ()> {
        let start = convert;
        let node = "uses";
        let start_range = convert.start_node(node)?;
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

        Ok((convert.subtract(start), Uses {use_imports}))
    }
}

/// Use import.
#[derive(Debug, Clone)]
pub struct UseImport {
    /// Namespace to import from.
    pub names: Vec<Arc<String>>,
    /// Function imports.
    pub fns: Vec<(Arc<String>, Option<Arc<String>>)>,
    /// The shared namespace alias.
    pub alias: Arc<String>,
}

impl UseImport {
    /// Create use import from meta data.
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, UseImport), ()> {
        let start = convert;
        let node = "use";
        let start_range = convert.start_node(node)?;
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
            names,
            fns,
            alias,
        }))
    }
}

/// Function.
#[derive(Debug, Clone)]
pub struct Function {
    /// The namespace of the function.
    pub namespace: Arc<Vec<Arc<String>>>,
    /// The name of the function.
    pub name: Arc<String>,
    /// The file which the function was loaded from.
    pub file: Arc<String>,
    /// The source code which the function is loaded from.
    pub source: Arc<String>,
    /// Function arguments.
    pub args: Vec<Arg>,
    /// Current object references.
    pub currents: Vec<Current>,
    /// Function block.
    pub block: Block,
    /// The return type of function.
    pub ret: Type,
    /// Whether local variable references has been resolved.
    pub resolved: Arc<sync::atomic::AtomicBool>,
    /// The range in source.
    pub source_range: Range,
    /// List of senders that receive function input by creating an in-type.
    pub senders: Arc<(
        sync::atomic::AtomicBool,
        sync::Mutex<Vec<sync::mpsc::Sender<Variable>>>
    )>,
}

impl Function {
    /// Creates function from meta data.
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

        let start = convert;
        let start_range = convert.start_node(node)?;
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
            } else if let Ok(_) = convert.start_node("ty") {
                // Ignore extra type information,
                // since this is only used by type checker.
                let range = convert.ignore();
                convert.update(range);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let mut name = name.ok_or(())?;
        let block = match expr {
            None => block.ok_or(())?,
            Some(expr) => {
                let source_range = expr.source_range();
                Block {
                    expressions: vec![Expression::Return(Box::new(expr))],
                    source_range
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
        let ret = ret.ok_or(())?;
        Ok((convert.subtract(start), Function {
            namespace: namespace.clone(),
            resolved: Arc::new(AtomicBool::new(false)),
            name,
            file: file.clone(),
            source: source.clone(),
            args,
            currents,
            block,
            ret,
            source_range: convert.source(start).unwrap(),
            senders: Arc::new((AtomicBool::new(false), Mutex::new(vec![]))),
        }))
    }

    /// Returns `true` if the function returns something.
    pub fn returns(&self) -> bool { self.ret != Type::Void }

    fn resolve_locals(&self, relative: usize, module: &Module, use_lookup: &UseLookup) {
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

/// Closure.
#[derive(Debug, Clone)]
pub struct Closure {
    /// The file where the closure was declared/created.
    pub file: Arc<String>,
    /// The source of the closure.
    pub source: Arc<String>,
    /// Closure arguments.
    pub args: Vec<Arg>,
    /// Current object references.
    pub currents: Vec<Current>,
    /// Closure expression.
    pub expr: Expression,
    /// The return type of the closure.
    pub ret: Type,
    /// The range in source.
    pub source_range: Range,
}

impl Closure {
    /// Create closure from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Closure), ()> {
        let start = convert;
        let start_range = convert.start_node(node)?;
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

        let ret = ret.ok_or(())?;
        let expr = expr.ok_or(())?;
        Ok((convert.subtract(start), Closure {
            file: file.clone(),
            source: source.clone(),
            args,
            currents,
            expr,
            ret,
            source_range: convert.source(start).unwrap(),
        }))
    }

    /// Returns `true` if the closure return something.
    pub fn returns(&self) -> bool { self.ret != Type::Void }

    fn resolve_locals(
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

/// Grab expression.
#[derive(Debug, Clone)]
pub struct Grab {
    /// Grab level.
    pub level: u16,
    /// The sub-expression to compute.
    pub expr: Expression,
    /// The range in source.
    pub source_range: Range,
}

impl Grab {
    /// Creates grab expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Grab), ()> {
        let start = convert;
        let node = "grab";
        let start_range = convert.start_node(node)?;
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
        let expr = expr.ok_or(())?;
        Ok((convert.subtract(start), Grab {
            level,
            expr,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn precompute(&self) -> Option<Variable> {
        self.expr.precompute()
    }

    fn resolve_locals(
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

/// Try-expression, catches run-time errors from sub-expression.
///
/// E.g. `try foo()`.
#[derive(Debug, Clone)]
pub struct TryExpr {
    /// Sub-expression to catch run-time errors from.
    pub expr: Expression,
    /// The range in source.
    pub source_range: Range,
}

impl TryExpr {
    /// Creates try expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, TryExpr), ()> {
        let start = convert;
        let node = "try_expr";
        let start_range = convert.start_node(node)?;
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

        let expr = expr.ok_or(())?;
        Ok((convert.subtract(start), TryExpr {
            expr,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn resolve_locals(
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

/// Function argument.
#[derive(Debug, Clone)]
pub struct Arg {
    /// The name of the argument.
    pub name: Arc<String>,
    /// The lifetime of the argument.
    pub lifetime: Option<Arc<String>>,
    /// The type of the argument.
    pub ty: Type,
    /// The range in source.
    pub source_range: Range,
    /// Whether the argument is mutable.
    pub mutable: bool,
}

impl Arg {
    /// Creates function argument from meta data.
    pub fn from_meta_data(mut convert: Convert, ignored: &mut Vec<Range>)
    -> Result<(Range, Arg), ()> {
        let start = convert;
        let node = "arg";
        let start_range = convert.start_node(node)?;
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

        let name = name.ok_or(())?;
        let ty = match ty {
            None => Type::Any,
            Some(ty) => ty
        };
        Ok((convert.subtract(start), Arg {
            name,
            lifetime,
            ty,
            source_range: convert.source(start).unwrap(),
            mutable,
        }))
    }
}

/// Current object reference.
///
/// This puts the current object into scope for a function.
#[derive(Debug, Clone)]
pub struct Current {
    /// The name of the current object.
    pub name: Arc<String>,
    /// The range in source.
    pub source_range: Range,
    /// Whether the current object is mutable.
    pub mutable: bool,
}

impl Current {
    /// Creates current object reference from meta data.
    pub fn from_meta_data(mut convert: Convert, ignored: &mut Vec<Range>)
    -> Result<(Range, Current), ()> {
        let start = convert;
        let node = "current";
        let start_range = convert.start_node(node)?;
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

        let name = name.ok_or(())?;
        Ok((convert.subtract(start), Current {
            name,
            source_range: convert.source(start).unwrap(),
            mutable,
        }))
    }
}

/// Block.
#[derive(Debug, Clone)]
pub struct Block {
    /// Sub-expression.
    pub expressions: Vec<Expression>,
    /// The range in source.
    pub source_range: Range,
}

impl Block {
    /// Creates block from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Block), ()> {
        let start = convert;
        let start_range = convert.start_node(node)?;
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
            expressions,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn resolve_locals(
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

/// Expression.
#[derive(Debug, Clone)]
pub enum Expression {
    /// Link expression.
    Link(Box<Link>),
    /// Object expression.
    Object(Box<Object>),
    /// Array expression.
    Array(Box<Array>),
    /// Array fill expression.
    ArrayFill(Box<ArrayFill>),
    /// Return expression.
    Return(Box<Expression>),
    /// Returns with value expression.
    ReturnVoid(Box<Range>),
    /// Break expression.
    Break(Box<Break>),
    /// Continue expression.
    Continue(Box<Continue>),
    /// Block expression.
    Block(Box<Block>),
    /// Go call expression.
    Go(Box<Go>),
    /// Call expression.
    Call(Box<Call>),
    /// Item expression.
    Item(Box<Item>),
    /// Binary operator expression.
    BinOp(Box<BinOpExpression>),
    /// Assignment expression.
    Assign(Box<Assign>),
    /// 4D vector expression.
    Vec4(Box<Vec4>),
    /// 4D matrix expression.
    Mat4(Box<Mat4>),
    /// For expression, e.g. `for i := 0; i < 10; i += 1 { ... }`.
    For(Box<For>),
    /// For-n expression.
    ForN(Box<ForN>),
    /// For-in expression.
    ForIn(Box<ForIn>),
    /// Sum for-n expression.
    Sum(Box<ForN>),
    /// Sum-in for-n expression.
    SumIn(Box<ForIn>),
    /// Component-wise 4D vector sum for-n-loop.
    SumVec4(Box<ForN>),
    /// Product for-n expression.
    Prod(Box<ForN>),
    /// Product-in for-n loop.
    ProdIn(Box<ForIn>),
    /// Component-wise 4D vector product for-n-loop.
    ProdVec4(Box<ForN>),
    /// Min for-n expression.
    Min(Box<ForN>),
    /// Min-in for-n expression.
    MinIn(Box<ForIn>),
    /// Max for-n expression.
    Max(Box<ForN>),
    /// Max-in for-n expression.
    MaxIn(Box<ForIn>),
    /// Sift for-n expression.
    Sift(Box<ForN>),
    /// Sift-in expression.
    SiftIn(Box<ForIn>),
    /// Any expression.
    Any(Box<ForN>),
    /// Any-in expression.
    AnyIn(Box<ForIn>),
    /// All-for expression.
    All(Box<ForN>),
    /// All-in expression.
    AllIn(Box<ForIn>),
    /// Link-for expression.
    LinkFor(Box<ForN>),
    /// Link-in expression.
    LinkIn(Box<ForIn>),
    /// If-expression.
    If(Box<If>),
    /// Compare expression.
    Compare(Box<Compare>),
    /// Unary operator expression.
    UnOp(Box<UnOpExpression>),
    /// Variable.
    ///
    /// This means it contains no members that depends on other expressions.
    Variable(Box<(Range, Variable)>),
    /// Try expression using `?`.
    Try(Box<Expression>),
    /// 4D vector swizzle expression.
    Swizzle(Box<Swizzle>),
    /// Closure expression.
    Closure(Arc<Closure>),
    /// Call closure expression.
    CallClosure(Box<CallClosure>),
    /// Grab expression.
    Grab(Box<Grab>),
    /// Try expression, e.g. `try x`.
    TryExpr(Box<TryExpr>),
    /// In-type expression.
    In(Box<In>),
}

// Required because the `Sync` impl of `Variable` is unsafe.
unsafe impl Sync for Expression {}

impl Expression {
    /// Creates expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Expression), ()> {
        let start = convert;
        let start_range = convert.start_node(node)?;
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
                result = Some(Expression::Link(Box::new(val)));
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
                result = Some(Expression::ReturnVoid(Box::new(
                    convert.source(start).unwrap())));
            } else if let Ok((range, val)) = Break::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Break(Box::new(val)));
            } else if let Ok((range, val)) = Continue::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Continue(Box::new(val)));
            } else if let Ok((range, val)) = Block::from_meta_data(
                    file, source, "block", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Block(Box::new(val)));
            } else if let Ok((range, val)) = Add::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(val.into_expression());
            } else if let Ok((range, val)) = UnOpExpression::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::UnOp(Box::new(val)));
            } else if let Ok((range, val)) = Mul::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(val.into_expression());
            } else if let Ok((range, val)) = Item::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Item(Box::new(val)));
            } else if let Ok((range, val)) = Norm::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(val.into_call_expr());
            } else if let Ok((range, val)) = convert.meta_string("text") {
                convert.update(range);
                result = Some(Expression::Variable(Box::new((
                    convert.source(start).unwrap(),
                    Variable::Str(val)
                ))));
            } else if let Ok((range, val)) = convert.meta_f64("num") {
                convert.update(range);
                result = Some(Expression::Variable(Box::new((
                    convert.source(start).unwrap(),
                    Variable::f64(val)
                ))));
            } else if let Ok((range, val)) = Vec4::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Vec4(Box::new(val)));
            } else if let Ok((range, val)) = Mat4::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Mat4(Box::new(val)));
            } else if let Ok((range, val)) = Vec4UnLoop::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(val.into_expression());
            } else if let Ok((range, val)) = convert.meta_bool("bool") {
                convert.update(range);
                result = Some(Expression::Variable(Box::new((
                    convert.source(start).unwrap(), Variable::bool(val)
                ))));
            } else if let Ok((range, val)) = convert.meta_string("color") {
                convert.update(range);
                if let Some((rgb, a)) = read_color::rgb_maybe_a(&mut val.chars()) {
                    let v = [
                            f32::from(rgb[0]) / 255.0,
                            f32::from(rgb[1]) / 255.0,
                            f32::from(rgb[2]) / 255.0,
                            f32::from(a.unwrap_or(255)) / 255.0
                        ];
                    result = Some(Expression::Variable(Box::new((range, Variable::Vec4(v)))));
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
                result = Some(Expression::Call(Box::new(val)));
            } else if let Ok((range, val)) = Call::named_from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Call(Box::new(val)));
            } else if let Ok((range, val)) = Assign::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Assign(Box::new(val)));
            } else if let Ok((range, val)) = For::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::For(Box::new(val)));
            } else if let Ok((range, val)) = ForIn::from_meta_data(
                    file, source, "for_in", convert, ignored) {
                convert.update(range);
                result = Some(Expression::ForIn(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "for_n", convert, ignored) {
                convert.update(range);
                result = Some(Expression::ForN(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "sum", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Sum(Box::new(val)));
            } else if let Ok((range, val)) = ForIn::from_meta_data(
                    file, source, "sum_in", convert, ignored) {
                convert.update(range);
                result = Some(Expression::SumIn(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "sum_vec4", convert, ignored) {
                convert.update(range);
                result = Some(Expression::SumVec4(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "prod", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Prod(Box::new(val)));
            } else if let Ok((range, val)) = ForIn::from_meta_data(
                    file, source, "prod_in", convert, ignored) {
                convert.update(range);
                result = Some(Expression::ProdIn(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "prod_vec4", convert, ignored) {
                convert.update(range);
                result = Some(Expression::ProdVec4(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "min", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Min(Box::new(val)));
            } else if let Ok((range, val)) = ForIn::from_meta_data(
                    file, source, "min_in", convert, ignored) {
                convert.update(range);
                result = Some(Expression::MinIn(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "max", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Max(Box::new(val)));
            } else if let Ok((range, val)) = ForIn::from_meta_data(
                    file, source, "max_in", convert, ignored) {
                convert.update(range);
                result = Some(Expression::MaxIn(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "sift", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Sift(Box::new(val)));
            } else if let Ok((range, val)) = ForIn::from_meta_data(
                    file, source, "sift_in", convert, ignored) {
                convert.update(range);
                result = Some(Expression::SiftIn(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "any", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Any(Box::new(val)));
            } else if let Ok((range, val)) = ForIn::from_meta_data(
                    file, source, "any_in", convert, ignored) {
                convert.update(range);
                result = Some(Expression::AnyIn(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "all", convert, ignored) {
                convert.update(range);
                result = Some(Expression::All(Box::new(val)));
            } else if let Ok((range, val)) = ForIn::from_meta_data(
                    file, source, "all_in", convert, ignored) {
                convert.update(range);
                result = Some(Expression::AllIn(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    file, source, "link_for", convert, ignored) {
                convert.update(range);
                result = Some(Expression::LinkFor(Box::new(val)));
            } else if let Ok((range, val)) = ForIn::from_meta_data(
                    file, source, "link_in", convert, ignored) {
                convert.update(range);
                result = Some(Expression::LinkIn(Box::new(val)));
            } else if let Ok((range, val)) = Loop::from_meta_data(
                    file, source, convert, ignored) {
                convert.update(range);
                result = Some(val.into_expression());
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
                    result = Some(Expression::Variable(Box::new((val.source_range, v))));
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

        let result = result.ok_or(())?;
        Ok((convert.subtract(start), result))
    }

    fn precompute(&self) -> Option<Variable> {
        use self::Expression::*;

        match *self {
            ArrayFill(ref array_fill) => array_fill.precompute(),
            Array(ref array) => array.precompute(),
            Object(ref obj) => obj.precompute(),
            Vec4(ref vec4) => vec4.precompute(),
            Link(ref link) => link.precompute(),
            Variable(ref range_var) => Some(range_var.1.clone()),
            _ => None
        }
    }

    /// Gets the range in source.
    pub fn source_range(&self) -> Range {
        use self::Expression::*;

        match *self {
            Link(ref link) => link.source_range,
            Object(ref obj) => obj.source_range,
            Array(ref arr) => arr.source_range,
            ArrayFill(ref arr_fill) => arr_fill.source_range,
            Return(ref expr) => expr.source_range(),
            ReturnVoid(ref range) => **range,
            Break(ref br) => br.source_range,
            Continue(ref c) => c.source_range,
            Block(ref bl) => bl.source_range,
            Go(ref go) => go.source_range,
            Call(ref call) => call.source_range,
            Item(ref it) => it.source_range,
            BinOp(ref binop) => binop.source_range,
            Assign(ref assign) => assign.source_range,
            Vec4(ref vec4) => vec4.source_range,
            Mat4(ref mat4) => mat4.source_range,
            For(ref for_expr) => for_expr.source_range,
            ForN(ref for_n_expr) => for_n_expr.source_range,
            ForIn(ref for_in_expr) => for_in_expr.source_range,
            Sum(ref for_n_expr) => for_n_expr.source_range,
            SumIn(ref for_in_expr) => for_in_expr.source_range,
            SumVec4(ref for_n_expr) => for_n_expr.source_range,
            Prod(ref for_n_expr) => for_n_expr.source_range,
            ProdIn(ref for_in_expr) => for_in_expr.source_range,
            ProdVec4(ref for_n_expr) => for_n_expr.source_range,
            Min(ref for_n_expr) => for_n_expr.source_range,
            MinIn(ref for_in_expr) => for_in_expr.source_range,
            Max(ref for_n_expr) => for_n_expr.source_range,
            MaxIn(ref for_in_expr) => for_in_expr.source_range,
            Sift(ref for_n_expr) => for_n_expr.source_range,
            SiftIn(ref for_in_expr) => for_in_expr.source_range,
            Any(ref for_n_expr) => for_n_expr.source_range,
            AnyIn(ref for_in_expr) => for_in_expr.source_range,
            All(ref for_n_expr) => for_n_expr.source_range,
            AllIn(ref for_in_expr) => for_in_expr.source_range,
            LinkFor(ref for_n_expr) => for_n_expr.source_range,
            LinkIn(ref for_in_expr) => for_in_expr.source_range,
            If(ref if_expr) => if_expr.source_range,
            Compare(ref comp) => comp.source_range,
            UnOp(ref unop) => unop.source_range,
            Variable(ref range_var) => range_var.0,
            Try(ref expr) => expr.source_range(),
            Swizzle(ref swizzle) => swizzle.source_range,
            Closure(ref closure) => closure.source_range,
            CallClosure(ref call) => call.source_range,
            Grab(ref grab) => grab.source_range,
            TryExpr(ref try_expr) => try_expr.source_range,
            In(ref in_expr) => in_expr.source_range,
        }
    }

    fn resolve_locals(
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
            Mat4(ref mat4) =>
                mat4.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            For(ref for_expr) =>
                for_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            ForN(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            ForIn(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Sum(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            SumIn(ref for_in_expr) =>
                for_in_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            SumVec4(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Prod(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            ProdIn(ref for_in_expr) =>
                for_in_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            ProdVec4(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Min(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            MinIn(ref for_in_expr) =>
                for_in_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Max(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            MaxIn(ref for_in_expr) =>
                for_in_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Sift(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            SiftIn(ref for_in_expr) =>
                for_in_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Any(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            AnyIn(ref for_in_expr) =>
                for_in_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            All(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            AllIn(ref for_in_expr) =>
                for_in_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            LinkFor(ref for_n_expr) =>
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            LinkIn(ref for_in_expr) =>
                for_in_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            If(ref if_expr) =>
                if_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Compare(ref comp) =>
                comp.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            UnOp(ref unop) =>
                unop.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Variable(_) => {}
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

/// Link expression, e.g. `link {a b}`.
#[derive(Debug, Clone)]
pub struct Link {
    /// Link item expressions.
    pub items: Vec<Expression>,
    /// The range in source.
    pub source_range: Range,
}

impl Link {
    /// Creates link expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Link), ()> {
        let start = convert;
        let node = "link";
        let start_range = convert.start_node(node)?;
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
            items,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn precompute(&self) -> Option<Variable> {
        let mut link = ::link::Link::new();
        for it in &self.items {
            if let Some(v) = it.precompute() {
                if link.push(&v).is_err() {return None};
            } else {
                return None;
            }
        }
        Some(Variable::Link(Box::new(link)))
    }

    fn resolve_locals(
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

/// Object expression.
#[derive(Debug, Clone)]
pub struct Object {
    /// Key-value pair expressions.
    pub key_values: Vec<(Arc<String>, Expression)>,
    /// The range in source.
    pub source_range: Range,
}

impl Object {
    /// Creates object expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Object), ()> {
        let start = convert;
        let node = "object";
        let start_range = convert.start_node(node)?;
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
            key_values,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn precompute(&self) -> Option<Variable> {
        let mut object: HashMap<_, _> = HashMap::new();
        for &(ref key, ref value) in &self.key_values {
            if let Some(v) = value.precompute() {
                object.insert(key.clone(), v);
            } else {
                return None;
            }
        }
        Some(Variable::Object(Arc::new(object)))
    }

    fn key_value_from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, (Arc<String>, Expression)), ()> {
        let start = convert;
        let node = "key_value";
        let start_range = convert.start_node(node)?;
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

        let key = key.ok_or(())?;
        let value = value.ok_or(())?;
        Ok((convert.subtract(start), (key, value)))
    }

    fn resolve_locals(
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

/// Array expression, e.g. `[a, b, c]`.
#[derive(Debug, Clone)]
pub struct Array {
    /// Array item expressions.
    pub items: Vec<Expression>,
    /// The range in source.
    pub source_range: Range,
}

impl Array {
    /// Creates array expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Array), ()> {
        let start = convert;
        let node = "array";
        let start_range = convert.start_node(node)?;
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
            items,
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

    fn resolve_locals(
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

/// Array fill expression, e.g. `[a; n]`.
#[derive(Debug, Clone)]
pub struct ArrayFill {
    /// The `a` in `[a; n]`.
    pub fill: Expression,
    /// The `n` in `[a; n]`.
    pub n: Expression,
    /// The range in source.
    pub source_range: Range,
}

impl ArrayFill {
    /// Creates array fill expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, ArrayFill), ()> {
        let start = convert;
        let node = "array_fill";
        let start_range = convert.start_node(node)?;
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

        let fill = fill.ok_or(())?;
        let n = n.ok_or(())?;
        Ok((convert.subtract(start), ArrayFill {
            fill,
            n,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn precompute(&self) -> Option<Variable> {
        if let Expression::Variable(ref range_var) = self.n {
            if let (_, Variable::F64(n, _)) = **range_var {
                if let Expression::Variable(ref x) = self.fill {
                    return Some(Variable::Array(Arc::new(vec![x.1.clone(); n as usize])));
                }
            }
        }
        None
    }

    fn resolve_locals(
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

/// Addition expression.
#[derive(Debug, Clone)]
pub struct Add {
    /// Item expressions.
    pub items: Vec<Expression>,
    /// Binary operators.
    pub ops: Vec<BinOp>,
    /// The range in source.
    pub source_range: Range,
}

impl Add {
    /// Creates addition expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Add), ()> {
        let start = convert;
        let node = "add";
        let start_range = convert.start_node(node)?;
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
            } else if let Ok((range, _)) = convert.meta_bool("⊻") {
                convert.update(range);
                ops.push(BinOp::Pow);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        if items.is_empty() {
            return Err(())
        }
        Ok((convert.subtract(start), Add {
            items,
            ops,
            source_range: convert.source(start).unwrap()
        }))
    }

    fn into_expression(mut self) -> Expression {
        if self.items.len() == 1 {
            self.items[0].clone()
        } else {
            let op = self.ops.pop().unwrap();
            let last = self.items.pop().unwrap();
            let source_range = self.source_range;
            Expression::BinOp(Box::new(BinOpExpression {
                op,
                left: self.into_expression(),
                right: last,
                source_range
            }))
        }
    }
}

/// Multiply expression.
#[derive(Debug, Clone)]
pub struct Mul {
    /// Item expressions.
    pub items: Vec<Expression>,
    /// Binary operators.
    pub ops: Vec<BinOp>,
    /// The range in source.
    pub source_range: Range,
}

impl Mul {
    /// Creates multiply expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Mul), ()> {
        let start = convert;
        let node = "mul";
        let start_range = convert.start_node(node)?;
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
                items.push(val.into_expression());
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

        if items.is_empty() {
            return Err(())
        }
        Ok((convert.subtract(start), Mul {
            items,
            ops,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn into_expression(mut self) -> Expression {
        if self.items.len() == 1 {
            self.items[0].clone()
        } else {
            let op = self.ops.pop().expect("Expected a binary operation");
            let last = self.items.pop().expect("Expected argument");
            let source_range = self.source_range;
            Expression::BinOp(Box::new(BinOpExpression {
                op,
                left: self.into_expression(),
                right: last,
                source_range,
            }))
        }
    }
}

/// Power expression.
#[derive(Debug, Clone)]
pub struct Pow {
    /// Base expression.
    ///
    /// This is the `x` in `x^a`.
    pub base: Expression,
    /// Exponent expression.
    ///
    /// This is the `x` in `a^x`.
    pub exp: Expression,
    /// The range in source.
    pub source_range: Range,
}

impl Pow {
    /// Creates power expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Pow), ()> {
        let start = convert;
        let node = "pow";
        let start_range = convert.start_node(node)?;
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

        let base = base.ok_or(())?;
        let exp = exp.ok_or(())?;
        Ok((convert.subtract(start), Pow {
            base,
            exp,
            source_range: convert.source(start).unwrap()
        }))
    }

    fn into_expression(self) -> Expression {
        Expression::BinOp(Box::new(BinOpExpression {
                op: BinOp::Pow,
                left: self.base,
                right: self.exp,
                source_range: self.source_range,
        }))
    }
}

/// Binary operator.
#[derive(Debug, Copy, Clone)]
pub enum BinOp {
    /// Addition operator (`+`).
    Add,
    /// Subtraction operator (`-`).
    Sub,
    /// Multiply operator (`*`).
    Mul,
    /// Dot product operator (`*.`).
    Dot,
    /// Cross product operator (`x`).
    Cross,
    /// Division operator (`/`).
    Div,
    /// Remainder operator (`%`).
    Rem,
    /// Power operator (`^`).
    Pow,
    /// Lazy OR operator (`||`).
    OrElse,
    /// Lazy AND operator (`&&`).
    AndAlso,
}

impl BinOp {
    /// Returns symbol of binary operator.
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

    /// Returns symbol of binary operator in boolean variant.
    pub fn symbol_bool(self) -> &'static str {
        match self {
            BinOp::Add => "or",
            BinOp::Mul => "and",
            _ => self.symbol()
        }
    }

    /// Returns the operator precedence level.
    /// Used to put parentheses in right places when printing out closures.
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

/// Unary operator.
#[derive(Debug, Copy, Clone)]
pub enum UnOp {
    /// Logical not.
    Not,
    /// Negation.
    Neg,
}

/// An item id.
///
/// This is the thing that's inside the square brackets, e.g. `foo[i]`.
#[derive(Debug, Clone)]
pub enum Id {
    /// A string.
    String(Range, Arc<String>),
    /// A number.
    F64(Range, f64),
    /// An expression.
    Expression(Expression),
}

impl Id {
    /// Gets the range in source.
    pub fn source_range(&self) -> Range {
        match *self {
            Id::String(range, _) => range,
            Id::F64(range, _) => range,
            Id::Expression(ref expr) => expr.source_range(),
        }
    }

    fn resolve_locals(
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

/// Item.
#[derive(Debug, Clone)]
pub struct Item {
    /// The name of item.
    pub name: Arc<String>,
    /// Dynamically resolved stack id.
    ///
    /// This is checked against the static stack id when
    /// the Cargo feature "debug_resolve" is enabled.
    pub stack_id: Cell<Option<usize>>,
    /// Statically resolved stack id.
    ///
    /// This is used when the Cargo feature "debug_resolve" is disabled.
    pub static_stack_id: Cell<Option<usize>>,
    /// Whether the item is a current object.
    pub current: bool,
    /// Whether there is a `?` after the item.
    pub try: bool,
    /// Item ids.
    pub ids: Vec<Id>,
    /// Stores indices of ids that should propagate errors.
    pub try_ids: Vec<usize>,
    /// The range in source.
    pub source_range: Range,
}

impl Item {
    /// Creates item from variable.
    pub fn from_variable(name: Arc<String>, source_range: Range) -> Item {
        Item {
            name,
            current: false,
            stack_id: Cell::new(None),
            static_stack_id: Cell::new(None),
            try: false,
            ids: vec![],
            try_ids: vec![],
            source_range
        }
    }

    /// Truncates item extra to a given length.
    fn trunc(&self, n: usize) -> Item {
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

    /// Creates item from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Item), ()> {
        let start = convert;
        let node = "item";
        let start_range = convert.start_node(node)?;
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

        let name = name.ok_or(())?;
        Ok((convert.subtract(start), Item {
            name,
            stack_id: Cell::new(None),
            static_stack_id: Cell::new(None),
            current,
            try,
            ids,
            try_ids,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn resolve_locals(
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
            if let Some(ref n) = *n {
                if **n == **self.name {
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

/// Go call.
#[derive(Debug, Clone)]
pub struct Go {
    /// Function call.
    pub call: Call,
    /// The range in source.
    pub source_range: Range,
}

impl Go {
    /// Creates go call from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Go), ()> {
        let start = convert;
        let node = "go";
        let start_range = convert.start_node(node)?;
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

        let call = call.ok_or(())?;
        Ok((convert.subtract(start), Go {
            call,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn resolve_locals(
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

/// Function call.
#[derive(Debug, Clone)]
pub struct Call {
    /// Alias.
    pub alias: Option<Arc<String>>,
    /// Name of function.
    pub name: Arc<String>,
    /// Arguments.
    pub args: Vec<Expression>,
    /// Function index.
    pub f_index: Cell<FnIndex>,
    /// A custom source, such as when calling a function inside a loaded module.
    pub custom_source: Option<Arc<String>>,
    /// The range in source.
    pub source_range: Range,
}

impl Call {
    /// Creates call from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Call), ()> {
        let start = convert;
        let node = "call";
        let start_range = convert.start_node(node)?;
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
                let mut peek = convert;
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

        let mut name = name.ok_or(())?;

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
            alias,
            name,
            args,
            f_index: Cell::new(FnIndex::None),
            custom_source: None,
            source_range: convert.source(start).unwrap(),
        }))
    }

    /// Creates named argument call from meta data.
    pub fn named_from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Call), ()> {
        let start = convert;
        let node = "named_call";
        let start_range = convert.start_node(node)?;
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
                if !name.is_empty() {
                    name.push('_');
                    name.push_str(&val);
                } else {
                    name.push_str(&val);
                    name.push('_');
                }
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "call_arg", convert, ignored) {
                let mut peek = convert;
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
            alias,
            name: Arc::new(name),
            args,
            f_index: Cell::new(FnIndex::None),
            custom_source: None,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn resolve_locals(
        &self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        use FnExternalRef;

        let st = stack.len();
        let f_index = if let Some(ref alias) = self.alias {
            if let Some(&i) = use_lookup.aliases.get(alias).and_then(|map| map.get(&self.name)) {
                match i {
                    FnAlias::Loaded(i) => FnIndex::Loaded(i as isize - relative as isize),
                    FnAlias::External(i) => {
                        let f = &module.ext_prelude[i];
                        if let Type::Void = f.p.ret {
                            FnIndex::ExternalVoid(FnExternalRef(f.f))
                        } else {
                            FnIndex::ExternalReturn(FnExternalRef(f.f))
                        }
                    }
                }
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

/// Closure call.
#[derive(Debug, Clone)]
pub struct CallClosure {
    /// The closure.
    pub item: Item,
    /// Closure argument expressions.
    pub args: Vec<Expression>,
    /// The range in source.
    pub source_range: Range,
}

impl CallClosure {
    /// Creates closure call from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, CallClosure), ()> {
        let start = convert;
        let node = "call_closure";
        let start_range = convert.start_node(node)?;
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
                let mut peek = convert;
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

        let item = item.ok_or(())?;
        Ok((convert.subtract(start), CallClosure {
            item,
            args,
            source_range: convert.source(start).unwrap(),
        }))
    }

    /// Creates named argument closure call from meta data.
    pub fn named_from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, CallClosure), ()> {
        let start = convert;
        let node = "named_call_closure";
        let start_range = convert.start_node(node)?;
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
                if !name.is_empty() {
                    name.push('_');
                    name.push_str(&val);
                } else {
                    name.push_str(&val);
                }
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "call_arg", convert, ignored) {
                let mut peek = convert;
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

        let mut item = item.ok_or(())?;
        {
            if item.ids.is_empty() {
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
            item,
            args,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn resolve_locals(
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

/// 4D vector norm.
#[derive(Debug, Clone)]
pub struct Norm {
    /// Expression argument.
    pub expr: Expression,
    /// The range in source.
    pub source_range: Range,
}

impl Norm {
    /// Creates 4D vector norm from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Norm), ()> {
        let start = convert;
        let node = "norm";
        let start_range = convert.start_node(node)?;
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

        let expr = expr.ok_or(())?;
        Ok((convert.subtract(start), Norm {
            expr,
            source_range: convert.source(start).unwrap()
        }))
    }

    fn into_call_expr(self) -> Expression {
        Expression::Call(Box::new(Call {
            alias: None,
            args: vec![self.expr],
            custom_source: None,
            f_index: Cell::new(FnIndex::None),
            name: Arc::new("norm".into()),
            source_range: self.source_range,
        }))
    }
}

/// Binary operator expression.
#[derive(Debug, Clone)]
pub struct BinOpExpression {
    /// Binary operator.
    pub op: BinOp,
    /// Left side expression.
    pub left: Expression,
    /// Right side expression.
    pub right: Expression,
    /// The range in source.
    pub source_range: Range,
}

impl BinOpExpression {
    fn resolve_locals(
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

/// Unary operator expression.
#[derive(Debug, Clone)]
pub struct UnOpExpression {
    /// Unary operator.
    pub op: UnOp,
    /// Expression argument.
    pub expr: Expression,
    /// The range in source.
    pub source_range: Range,
}

impl UnOpExpression {
    /// Creates unary operator expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, UnOpExpression), ()> {
        let start = convert;
        let node = "unop";
        let start_range = convert.start_node(node)?;
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

        let unop = unop.ok_or(())?;
        let expr = expr.ok_or(())?;
        Ok((convert.subtract(start), UnOpExpression {
            op: unop,
            expr,
            source_range: convert.source(start).unwrap()
        }))
    }

    fn resolve_locals(
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

/// Assignment expression.
#[derive(Debug, Clone)]
pub struct Assign {
    /// Assignment operator.
    pub op: AssignOp,
    /// Left side expression.
    pub left: Expression,
    /// Right side expression.
    pub right: Expression,
    /// The range in source.
    pub source_range: Range,
}

impl Assign {
    /// Creates assignment expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Assign), ()> {
        let start = convert;
        let node = "assign";
        let start_range = convert.start_node(node)?;
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

        let op = op.ok_or(())?;
        let left = left.ok_or(())?;
        let right = right.ok_or(())?;
        Ok((convert.subtract(start), Assign {
            op,
            left,
            right,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn resolve_locals(
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
            if item.ids.is_empty() && self.op == AssignOp::Assign {
                stack.push(Some(item.name.clone()));
                return;
            }
        }

        self.left.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }
}

/// Assignment operator.
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
    /// Returns symbol of assignment operator.
    pub fn symbol(self) -> &'static str {
        use self::AssignOp::*;

        match self {
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

/// 4D matrix expression.
#[derive(Debug, Clone)]
pub struct Mat4 {
    /// Row vector argument expressions.
    pub args: Vec<Expression>,
    /// The range in source.
    pub source_range: Range,
}

impl Mat4 {
    /// Creates 4D matrix from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Mat4), ()> {
        let start = convert;
        let node = "mat4";
        let start_range = convert.start_node(node)?;
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
                file, source, "ex", convert, ignored) {
                convert.update(range);
                x = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "ey", convert, ignored) {
                convert.update(range);
                y = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "ez", convert, ignored) {
                convert.update(range);
                z = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "ew", convert, ignored) {
                convert.update(range);
                w = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let x = x.ok_or(())?;
        let y = y.unwrap_or_else(|| Expression::Variable(Box::new((
            Range::empty(0), Variable::Vec4([0.0, 1.0, 0.0, 0.0])))));
        let z = z.unwrap_or_else(|| Expression::Variable(Box::new((
            Range::empty(0), Variable::Vec4([0.0, 0.0, 1.0, 0.0])))));
        let w = w.unwrap_or_else(|| Expression::Variable(Box::new((
            Range::empty(0), Variable::Vec4([0.0, 0.0, 0.0, 1.0])))));
        Ok((convert.subtract(start), Mat4 {
            args: vec![x, y, z, w],
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn resolve_locals(
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
            stack.push(None);
        }
        stack.truncate(st);
    }
}

/// 4D vector expression.
#[derive(Debug, Clone)]
pub struct Vec4 {
    /// Component expressions.
    pub args: Vec<Expression>,
    /// The range in source.
    pub source_range: Range,
}

impl Vec4 {
    /// Creates 4D vector from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Vec4), ()> {
        let start = convert;
        let node = "vec4";
        let start_range = convert.start_node(node)?;
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

        let x = x.ok_or(())?;
        let y = y.unwrap_or_else(||
            Expression::Variable(Box::new((Range::empty(0), Variable::f64(0.0)))));
        let z = z.unwrap_or_else(||
            Expression::Variable(Box::new((Range::empty(0), Variable::f64(0.0)))));
        let w = w.unwrap_or_else(||
            Expression::Variable(Box::new((Range::empty(0), Variable::f64(0.0)))));
        Ok((convert.subtract(start), Vec4 {
            args: vec![x, y, z, w],
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn precompute(&self) -> Option<Variable> {
        let mut v: [f32; 4] = [0.0; 4];
        for i in 0..self.args.len().min(4) {
            if let Some(val) = self.args[i].precompute() {
                if let Variable::F64(val, _) = val {
                    v[i] = val as f32;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        Some(Variable::Vec4(v))
    }

    fn resolve_locals(
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

/// 4D vector unloop expression.
#[derive(Debug, Clone)]
pub struct Vec4UnLoop {
    /// Name of the variable.
    pub name: Arc<String>,
    /// Expression of the loop.
    pub expr: Expression,
    /// The length of the loop.
    pub len: u8,
    /// The range in source.
    pub source_range: Range,
}

impl Vec4UnLoop {
    /// Creates 4D vector-unloop from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Vec4UnLoop), ()> {
        let start = convert;
        let node = "vec4_un_loop";
        let start_range = convert.start_node(node)?;
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

        let name = name.ok_or(())?;
        let expr = expr.ok_or(())?;
        Ok((convert.subtract(start), Vec4UnLoop {
            name,
            expr,
            len,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn into_expression(self) -> Expression {
        let source_range = self.source_range;

        let zero = || Expression::Variable(Box::new((source_range, Variable::f64(0.0))));

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

        Expression::Vec4(Box::new(Vec4 {
            args: vec![replace_0, replace_1, replace_2, replace_3],
            source_range,
        }))
    }
}

/// Swizzle expression.
#[derive(Debug, Clone)]
pub struct Swizzle {
    /// First component swizzle.
    pub sw0: usize,
    /// Second component swizzle.
    pub sw1: usize,
    /// Third component swizzle.
    pub sw2: Option<usize>,
    /// Fourth component swizzle.
    pub sw3: Option<usize>,
    /// 4D vector expression.
    pub expr: Expression,
    /// The range in source.
    pub source_range: Range,
}

impl Swizzle {
    /// Create swizzle from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Swizzle), ()> {
        let start = convert;
        let node = "swizzle";
        let start_range = convert.start_node(node)?;
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

        let sw0 = sw0.ok_or(())?;
        let sw1 = sw1.ok_or(())?;
        let expr = expr.ok_or(())?;
        Ok((convert.subtract(start), Swizzle {
            sw0,
            sw1,
            sw2,
            sw3,
            expr,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn len(&self) -> usize {
        2 + if self.sw2.is_some() { 1 } else { 0 } +
            if self.sw3.is_some() { 1 } else { 0 }
    }
}

/// Component swizzle expression.
#[derive(Debug, Clone)]
pub struct Sw {
    /// The component index of the swizzle.
    pub ind: usize,
    /// The range in source.
    pub source_range: Range,
}

impl Sw {
    /// Creates component swizzle from meta data.
    pub fn from_meta_data(
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Sw), ()> {
        let start = convert;
        let start_range = convert.start_node(node)?;
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

        let ind = ind.ok_or(())?;
        Ok((convert.subtract(start), Sw {
            ind,
            source_range: convert.source(start).unwrap(),
        }))
    }
}

/// For-expression.
#[derive(Debug, Clone)]
pub struct For {
    /// The initial expression.
    pub init: Expression,
    /// Expression evaluated for determining whether to continue or not.
    pub cond: Expression,
    /// Expression evaluated at each step.
    pub step: Expression,
    /// Block expression.
    pub block: Block,
    /// Loop label.
    pub label: Option<Arc<String>>,
    /// The range in source.
    pub source_range: Range,
}

impl For {
    /// Creates For-expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, For), ()> {
        let start = convert;
        let node = "for";
        let start_range = convert.start_node(node)?;
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

        let init = init.ok_or(())?;
        let cond = cond.ok_or(())?;
        let step = step.ok_or(())?;
        let block = block.ok_or(())?;
        Ok((convert.subtract(start), For {
            init,
            cond,
            step,
            block,
            label,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn resolve_locals(
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

/// For-In expression.
#[derive(Debug, Clone)]
pub struct ForIn {
    /// Name of the loop variable.
    pub name: Arc<String>,
    /// The in-type expression to read from.
    pub iter: Expression,
    /// Block expression.
    pub block: Block,
    /// Loop label.
    pub label: Option<Arc<String>>,
    /// The range in source.
    pub source_range: Range,
}

impl ForIn {
    /// Creates For-In expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, ForIn), ()> {
        let start = convert;
        let start_range = convert.start_node(node)?;
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
        let mut iter: Option<Expression> = None;
        let mut block: Option<Block> = None;
        let mut label: Option<Arc<String>> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                file, source, "iter", convert, ignored) {
                convert.update(range);
                iter = Some(val);
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

        let name = name.ok_or(())?;
        let iter = iter.ok_or(())?;
        let block = block.ok_or(())?;
        Ok((convert.subtract(start), ForIn {
            name, iter, block, label,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn resolve_locals(
        &self, relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.iter.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
        stack.push(Some(self.name.clone()));
        self.block.resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }
}

/// For-N expression.
#[derive(Debug, Clone)]
pub struct ForN {
    /// Name of index variable.
    pub name: Arc<String>,
    /// Start expression.
    ///
    /// This is evaluated before starting the loop.
    pub start: Option<Expression>,
    /// End expression.
    ///
    /// This is evaluated before starting the loop.
    pub end: Expression,
    /// Bloc expression.
    pub block: Block,
    /// Loop label.
    pub label: Option<Arc<String>>,
    /// The range in source.
    pub source_range: Range,
}

impl ForN {
    /// Creates For-N expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, ForN), ()> {
        let start = convert;
        let start_range = convert.start_node(node)?;
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
        if indices.is_empty() { return Err(()); }

        let name: Arc<String> = indices[0].0.clone();
        let start_expr = indices[0].1.clone();
        let mut end_expr = indices[0].2.clone();

        if indices.len() > 1 {
            let (_, new_for_n) = ForN::create(
                node,
                range,
                source_range,
                None,
                &indices[1..],
                block
            )?;
            block = Some(Block {
                source_range,
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

        let block = block.ok_or(())?;

        // Infer list length from index.
        if end_expr.is_none() {
            end_expr = infer_len::infer(&block, &name);
        }

        let end_expr = end_expr.ok_or(())?;
        Ok((range, ForN {
            name,
            start: start_expr,
            end: end_expr,
            block,
            label,
            source_range,
        }))
    }

    fn resolve_locals(
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

/// Loop expression.
#[derive(Debug, Clone)]
pub struct Loop {
    /// Block expression.
    pub block: Block,
    /// Loop label.
    pub label: Option<Arc<String>>,
    /// The range in source.
    pub source_range: Range,
}

impl Loop {
    /// Creates loop expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Loop), ()> {
        let start = convert;
        let node = "loop";
        let start_range = convert.start_node(node)?;
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

        let block = block.ok_or(())?;
        Ok((convert.subtract(start), Loop {
            block,
            label,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn into_expression(self) -> Expression {
        let source_range = self.source_range;
        Expression::For(Box::new(For {
            block: self.block,
            label: self.label,
            init: Expression::Block(Box::new(Block {
                expressions: vec![],
                source_range,
            })),
            step: Expression::Block(Box::new(Block {
                expressions: vec![],
                source_range,
            })),
            cond: Expression::Variable(Box::new((source_range, Variable::bool(true)))),
            source_range,
        }))
    }
}

/// Break expression.
#[derive(Debug, Clone)]
pub struct Break {
    /// Loop label.
    pub label: Option<Arc<String>>,
    /// The range in source.
    pub source_range: Range,
}

impl Break {
    /// Creates break expression from meta data.
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Break), ()> {
        let start = convert;
        let node = "break";
        let start_range = convert.start_node(node)?;
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
            label,
            source_range: convert.source(start).unwrap(),
        }))
    }
}

/// Continue expression.
#[derive(Debug, Clone)]
pub struct Continue {
    /// The loop label.
    pub label: Option<Arc<String>>,
    /// The range in source.
    pub source_range: Range,
}

impl Continue {
    /// Creates continue expression from meta data.
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Continue), ()> {
        let start = convert;
        let node = "continue";
        let start_range = convert.start_node(node)?;
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
            label,
            source_range: convert.source(start).unwrap(),
        }))
    }
}

/// If-expression.
#[derive(Debug, Clone)]
pub struct If {
    /// Condition.
    pub cond: Expression,
    /// True block.
    pub true_block: Block,
    /// Else-if conditions.
    pub else_if_conds: Vec<Expression>,
    /// Else-if blocks.
    pub else_if_blocks: Vec<Block>,
    /// Else block.
    pub else_block: Option<Block>,
    /// The range in source.
    pub source_range: Range,
}

impl If {
    /// Creates if-expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, If), ()> {
        let start = convert;
        let node = "if";
        let start_range = convert.start_node(node)?;
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

        let cond = cond.ok_or(())?;
        let true_block = true_block.ok_or(())?;
        Ok((convert.subtract(start), If {
            cond,
            true_block,
            else_if_conds,
            else_if_blocks,
            else_block,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn resolve_locals(
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

/// Compare expression.
#[derive(Debug, Clone)]
pub struct Compare {
    /// Comparison operator.
    pub op: CompareOp,
    /// Left side of expression.
    pub left: Expression,
    /// Right side of expression.
    pub right: Expression,
    /// The range in source.
    pub source_range: Range,
}

impl Compare {
    /// Creates compare expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Compare), ()> {
        let start = convert;
        let node = "compare";
        let start_range = convert.start_node(node)?;
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

        let op = op.ok_or(())?;
        let left = left.ok_or(())?;
        let right = right.ok_or(())?;
        Ok((convert.subtract(start), Compare {
            op,
            left,
            right,
            source_range: convert.source(start).unwrap(),
        }))
    }

    fn resolve_locals(
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

/// Comparison operator.
#[derive(Debug, Clone, Copy)]
pub enum CompareOp {
    /// Less.
    Less,
    /// Less or equal.
    LessOrEqual,
    /// Greater.
    Greater,
    /// Greater or equal.
    GreaterOrEqual,
    /// Equal.
    Equal,
    /// Not equal.
    NotEqual,
}

impl CompareOp {
    /// Returns symbol for the comparison operator.
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

/// Stores `in <function>` expression.
#[derive(Debug, Clone)]
pub struct In {
    /// Alias, e.g. `in foo::my_function`.
    pub alias: Option<Arc<String>>,
    /// Name of function.
    pub name: Arc<String>,
    /// Function index.
    pub f_index: Cell<FnIndex>,
    /// Range is source file.
    pub source_range: Range,
}

impl In {
    /// Creates in expression from meta data.
    pub fn from_meta_data(
        node: &'static str,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, In), ()> {
        let start = convert;
        let start_range = convert.start_node(node)?;
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

        let name = name.ok_or(())?;
        Ok((convert.subtract(start), In {
            alias,
            name,
            f_index: Cell::new(FnIndex::None),
            source_range: convert.source(start).unwrap()
        }))
    }

    fn resolve_locals(
        &self,
        relative: usize,
        module: &Module,
        use_lookup: &UseLookup
    ) {
        use FnExternalRef;

        let f_index = if let Some(ref alias) = self.alias {
            if let Some(&i) = use_lookup.aliases.get(alias).and_then(|map| map.get(&self.name)) {
                match i {
                    FnAlias::Loaded(i) => FnIndex::Loaded(i as isize - relative as isize),
                    FnAlias::External(i) => {
                        let f = &module.ext_prelude[i];
                        if let Type::Void = f.p.ret {
                            FnIndex::ExternalVoid(FnExternalRef(f.f))
                        } else {
                            FnIndex::ExternalReturn(FnExternalRef(f.f))
                        }
                    }
                }
            } else {
                FnIndex::None
            }
        } else {
            module.find_function(&self.name, relative)
        };
        self.f_index.set(f_index);
    }
}
