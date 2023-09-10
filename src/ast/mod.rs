//! Dyon Abstract Syntax Tree (AST).

use piston_meta::bootstrap::Convert;
use piston_meta::MetaData;
use range::Range;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::{self, Arc};

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
    module: &mut Module,
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
            Function::from_meta_data(&namespace, &file, &source, "fn", convert, ignored)
        {
            convert.update(range);
            module.register(function);
        } else if convert.remaining_data_len() > 0 {
            return Err(());
        } else {
            break;
        }
    }
    let mut new_functions = module.functions.clone();
    for i in 0..new_functions.len() {
        new_functions[i].resolve_locals(i, module, &use_lookup);
    }
    module.functions = new_functions;
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
    fn default() -> UseLookup {
        UseLookup::new()
    }
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
            if !use_import.fns.is_empty() {
                continue;
            }
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
            if use_import.fns.is_empty() {
                continue;
            }
            if !aliases.contains_key(&use_import.alias) {
                aliases.insert(use_import.alias.clone(), HashMap::new());
            }
            let fns = aliases.get_mut(&use_import.alias).unwrap();
            for use_fn in &use_import.fns {
                for (i, f) in module.functions.iter().enumerate().rev() {
                    if *f.namespace != use_import.names {
                        continue;
                    }
                    if f.name == use_fn.0 {
                        fns.insert(
                            use_fn.1.as_ref().unwrap_or(&use_fn.0).clone(),
                            FnAlias::Loaded(i),
                        );
                    } else if f.name.len() > use_fn.0.len()
                        && f.name.starts_with(&**use_fn.0)
                        && f.name.as_bytes()[use_fn.0.len()] == b'('
                    {
                        // A function with mutable information.
                        let mut name: Arc<String> = use_fn.1.as_ref().unwrap_or(&use_fn.0).clone();
                        Arc::make_mut(&mut name).push_str(&f.name.as_str()[use_fn.0.len()..]);
                        fns.insert(name, FnAlias::Loaded(i));
                    }
                }
                for (i, f) in module.ext_prelude.iter().enumerate().rev() {
                    if *f.namespace != use_import.names {
                        continue;
                    }
                    if f.name == use_fn.0 {
                        fns.insert(
                            use_fn.1.as_ref().unwrap_or(&use_fn.0).clone(),
                            FnAlias::External(i),
                        );
                    } else if f.name.len() > use_fn.0.len()
                        && f.name.starts_with(&**use_fn.0)
                        && f.name.as_bytes()[use_fn.0.len()] == b'('
                    {
                        // A function with mutable information.
                        let mut name: Arc<String> = use_fn.1.as_ref().unwrap_or(&use_fn.0).clone();
                        Arc::make_mut(&mut name).push_str(&f.name.as_str()[use_fn.0.len()..]);
                        fns.insert(f.name.clone(), FnAlias::External(i));
                    }
                }
            }
        }
        UseLookup { aliases }
    }

    /// This is called from lifetime/type checker.
    /// Here, external functions are treated as loaded.
    pub fn from_uses_prelude(uses: &Uses, prelude: &Prelude) -> UseLookup {
        let mut aliases = HashMap::new();
        // First, add all glob imports.
        for use_import in &uses.use_imports {
            if !use_import.fns.is_empty() {
                continue;
            }
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
            if use_import.fns.is_empty() {
                continue;
            }
            if !aliases.contains_key(&use_import.alias) {
                aliases.insert(use_import.alias.clone(), HashMap::new());
            }
            let fns = aliases.get_mut(&use_import.alias).unwrap();
            for use_fn in &use_import.fns {
                for (i, f) in prelude.namespaces.iter().enumerate().rev() {
                    if *f.0 != use_import.names {
                        continue;
                    }
                    if f.1 == use_fn.0 {
                        fns.insert(
                            use_fn.1.as_ref().unwrap_or(&use_fn.0).clone(),
                            FnAlias::Loaded(i),
                        );
                    } else if f.1.len() > use_fn.0.len()
                        && f.1.starts_with(&**use_fn.0)
                        && f.1.as_bytes()[use_fn.0.len()] == b'('
                    {
                        // A function with mutable information.
                        let mut name: Arc<String> = use_fn.1.as_ref().unwrap_or(&use_fn.0).clone();
                        Arc::make_mut(&mut name).push_str(&f.1.as_str()[use_fn.0.len()..]);
                        fns.insert(name, FnAlias::Loaded(i));
                    }
                }
            }
        }
        UseLookup { aliases }
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
        ignored: &mut Vec<Range>,
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

        Ok((
            convert.subtract(start),
            Namespace {
                names: Arc::new(names),
            },
        ))
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
        ignored: &mut Vec<Range>,
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

        Ok((convert.subtract(start), Uses { use_imports }))
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
        ignored: &mut Vec<Range>,
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
        Ok((convert.subtract(start), UseImport { names, fns, alias }))
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
    /// Lazy invariants.
    pub lazy_inv: Vec<Vec<Lazy>>,
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
        sync::Mutex<Vec<sync::mpsc::Sender<Variable>>>,
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Function), ()> {
        use std::sync::atomic::AtomicBool;
        use std::sync::Mutex;

        let start = convert;
        let start_range = convert.start_node(node)?;
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
        let mut args: Vec<Arg> = vec![];
        let mut currents: Vec<Current> = vec![];
        let mut block: Option<Block> = None;
        let mut expr: Option<Expression> = None;
        let mut ret: Option<Type> = None;
        let mut lazy_inv: Vec<Vec<Lazy>> = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else if let Ok((range, val, lazy)) =
                Arg::from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                args.push(val);
                lazy_inv.push(lazy);
            } else if let Ok((range, val)) = Current::from_meta_data(convert, ignored) {
                convert.update(range);
                currents.push(val);
            } else if let Ok((range, val)) = convert.meta_bool("returns") {
                convert.update(range);
                ret = Some(if val { Type::Any } else { Type::Void })
            } else if let Ok((range, val)) = Type::from_meta_data("ret_type", convert, ignored) {
                convert.update(range);
                ret = Some(val);
            } else if let Ok((range, val)) =
                Block::from_meta_data(file, source, "block", convert, ignored)
            {
                convert.update(range);
                block = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "expr", convert, ignored)
            {
                convert.update(range);
                expr = Some(val);
                ret = Some(Type::Any);
            } else if convert.start_node("ty").is_ok() {
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
                    source_range,
                }
            }
        };
        let mutable_args = args.iter().any(|arg| arg.mutable);
        if mutable_args {
            let mut name_plus_args = String::from(&**name);
            name_plus_args.push('(');
            let mut first = true;
            for arg in &args {
                if !first {
                    name_plus_args.push(',');
                }
                name_plus_args.push_str(if arg.mutable { "mut" } else { "_" });
                first = false;
            }
            name_plus_args.push(')');
            name = Arc::new(name_plus_args);
        }
        let ret = ret.ok_or(())?;
        // Remove empty lazy invariants.
        while let Some(true) = lazy_inv.last().map(|lz| lz.is_empty()) {
            lazy_inv.pop();
        }
        Ok((
            convert.subtract(start),
            Function {
                namespace: namespace.clone(),
                resolved: Arc::new(AtomicBool::new(false)),
                name,
                file: file.clone(),
                source: source.clone(),
                args,
                lazy_inv,
                currents,
                block,
                ret,
                source_range: convert.source(start).unwrap(),
                senders: Arc::new((AtomicBool::new(false), Mutex::new(vec![]))),
            },
        ))
    }

    /// Returns `true` if the function returns something.
    pub fn returns(&self) -> bool {
        self.ret != Type::Void
    }

    fn resolve_locals(&mut self, relative: usize, module: &Module, use_lookup: &UseLookup) {
        use std::sync::atomic::Ordering;

        // Ensure sequential order just to be safe.
        if self.resolved.load(Ordering::SeqCst) {
            return;
        }
        let mut stack: Vec<Option<Arc<String>>> = vec![];
        let mut closure_stack: Vec<usize> = vec![];
        if self.returns() {
            // Use return type because it has the same name.
            stack.push(Some(crate::runtime::RETURN_TYPE.clone()));
        }
        for arg in &self.args {
            stack.push(Some(arg.name.clone()));
        }
        for current in &self.currents {
            stack.push(Some(current.name.clone()));
        }
        self.block
            .resolve_locals(relative, &mut stack, &mut closure_stack, module, use_lookup);
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
        ignored: &mut Vec<Range>,
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
            } else if let Ok((range, val, _)) = Arg::from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                args.push(val);
            } else if let Ok((range, val)) = Current::from_meta_data(convert, ignored) {
                convert.update(range);
                currents.push(val);
            } else if let Ok((range, val)) = convert.meta_bool("returns") {
                convert.update(range);
                ret = Some(if val { Type::Any } else { Type::Void })
            } else if let Ok((range, val)) = Type::from_meta_data("ret_type", convert, ignored) {
                convert.update(range);
                ret = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "expr", convert, ignored)
            {
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
        Ok((
            convert.subtract(start),
            Closure {
                file: file.clone(),
                source: source.clone(),
                args,
                currents,
                expr,
                ret,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    /// Returns `true` if the closure return something.
    pub fn returns(&self) -> bool {
        self.ret != Type::Void
    }

    fn resolve_locals(
        &mut self,
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
            // Use return type because it has the same name.
            stack.push(Some(crate::runtime::RETURN_TYPE.clone()));
        }
        for arg in &self.args {
            stack.push(Some(arg.name.clone()));
        }
        for current in &self.currents {
            stack.push(Some(current.name.clone()));
        }
        self.expr
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
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
        ignored: &mut Vec<Range>,
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "expr", convert, ignored)
            {
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
        Ok((
            convert.subtract(start),
            Grab {
                level,
                expr,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn precompute(&self) -> Option<Variable> {
        self.expr.precompute()
    }

    fn resolve_locals(
        &mut self,
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
        self.expr
            .resolve_locals(relative, &mut tmp_stack, closure_stack, module, use_lookup)
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
        ignored: &mut Vec<Range>,
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "expr", convert, ignored)
            {
                convert.update(range);
                expr = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let expr = expr.ok_or(())?;
        Ok((
            convert.subtract(start),
            TryExpr {
                expr,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        self.expr
            .resolve_locals(relative, stack, closure_stack, module, use_lookup)
    }
}

/// Lazy invariant.
#[derive(Debug, Clone, PartialEq)]
pub enum Lazy {
    /// Return a variable if result of argument is equal to the variable.
    Variable(Variable),
    /// Unwrap the Ok result.
    UnwrapOk,
    /// Unwrap the Err result.
    UnwrapErr,
    /// Unwrap the Some option.
    UnwrapSome,
}

/// This is requires because `UnsafeRef(*mut Variable)` can not be sent across threads.
/// The lack of `UnsafeRef` variant when sending across threads is guaranteed at language level.
/// The interior of `UnsafeRef` can not be accessed outside this library.
unsafe impl Sync for Lazy {}

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
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Arg, Vec<Lazy>), ()> {
        let start = convert;
        let node = "arg";
        let start_range = convert.start_node(node)?;
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
        let mut lifetime: Option<Arc<String>> = None;
        let mut ty: Option<Type> = None;
        let mut mutable = false;
        let mut lazy: Vec<Lazy> = vec![];
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
            } else if let Ok((range, val)) = Type::from_meta_data("type", convert, ignored) {
                convert.update(range);
                ty = Some(val);
            } else if let Ok((range, val)) = Grab::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                if let Some(val) = val.precompute() {
                    lazy.push(Lazy::Variable(val));
                } else {
                    return Err(());
                }
            } else if let Ok((range, _)) = convert.meta_bool("ok(_)") {
                convert.update(range);
                lazy.push(Lazy::UnwrapOk);
            } else if let Ok((range, _)) = convert.meta_bool("err(_)") {
                convert.update(range);
                lazy.push(Lazy::UnwrapErr);
            } else if let Ok((range, _)) = convert.meta_bool("some(_)") {
                convert.update(range);
                lazy.push(Lazy::UnwrapSome);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let name = name.ok_or(())?;
        let ty = match ty {
            None => Type::Any,
            Some(ty) => ty,
        };
        Ok((
            convert.subtract(start),
            Arg {
                name,
                lifetime,
                ty,
                source_range: convert.source(start).unwrap(),
                mutable,
            },
            lazy,
        ))
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
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Current), ()> {
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
            } else if let Ok((range, _)) = Type::from_meta_data("type", convert, ignored) {
                convert.update(range);
                // Just ignore type for now.
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let name = name.ok_or(())?;
        Ok((
            convert.subtract(start),
            Current {
                name,
                source_range: convert.source(start).unwrap(),
                mutable,
            },
        ))
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Block), ()> {
        let start = convert;
        let start_range = convert.start_node(node)?;
        convert.update(start_range);

        let mut expressions = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "expr", convert, ignored)
            {
                convert.update(range);
                expressions.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((
            convert.subtract(start),
            Block {
                expressions,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for expr in &mut self.expressions {
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
    /// Call external function.
    CallVoid(Box<CallVoid>),
    /// Call external function that returns something.
    CallReturn(Box<CallReturn>),
    /// Call external function with lazy invariant.
    CallLazy(Box<CallLazy>),
    /// Call loaded function.
    CallLoaded(Box<CallLoaded>),
    /// Binary operator.
    CallBinOp(Box<CallBinOp>),
    /// Unary operator.
    CallUnOp(Box<CallUnOp>),
    /// Item expression.
    Item(Box<Item>),
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Expression), ()> {
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
            } else if let Ok((range, val)) = Link::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Link(Box::new(val)));
            } else if let Ok((range, val)) = Object::from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Object(Box::new(val)));
            } else if let Ok((range, val)) = Array::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Array(Box::new(val)));
            } else if let Ok((range, val)) =
                ArrayFill::from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::ArrayFill(Box::new(val)));
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "return", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Return(Box::new(val)));
            } else if let Ok((range, _)) = convert.meta_bool("return_void") {
                convert.update(range);
                result = Some(Expression::ReturnVoid(Box::new(
                    convert.source(start).unwrap(),
                )));
            } else if let Ok((range, val)) = Break::from_meta_data(convert, ignored) {
                convert.update(range);
                result = Some(Expression::Break(Box::new(val)));
            } else if let Ok((range, val)) = Continue::from_meta_data(convert, ignored) {
                convert.update(range);
                result = Some(Expression::Continue(Box::new(val)));
            } else if let Ok((range, val)) =
                Block::from_meta_data(file, source, "block", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Block(Box::new(val)));
            } else if let Ok((range, val)) =
                BinOpSeq::from_meta_data(file, source, "add", convert, ignored)
            {
                convert.update(range);
                result = Some(val.into_expression());
            } else if let Ok((range, val)) =
                UnOpExpression::from_meta_data("not", file, source, convert, ignored)
            {
                convert.update(range);
                result = Some(val);
            } else if let Ok((range, val)) =
                BinOpSeq::from_meta_data(file, source, "mul", convert, ignored)
            {
                convert.update(range);
                result = Some(val.into_expression());
            } else if let Ok((range, val)) =
                BinOpSeq::from_meta_data(file, source, "compare", convert, ignored)
            {
                convert.update(range);
                result = Some(val.into_expression());
            } else if let Ok((range, val)) = Item::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Item(Box::new(val)));
            } else if let Ok((range, val)) = Norm::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                result = Some(val.into_call_expr());
            } else if let Ok((range, val)) = convert.meta_string("text") {
                convert.update(range);
                result = Some(Expression::Variable(Box::new((
                    convert.source(start).unwrap(),
                    Variable::Str(val),
                ))));
            } else if let Ok((range, val)) = convert.meta_f64("num") {
                convert.update(range);
                result = Some(Expression::Variable(Box::new((
                    convert.source(start).unwrap(),
                    Variable::f64(val),
                ))));
            } else if let Ok((range, val)) = Vec4::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Vec4(Box::new(val)));
            } else if let Ok((range, val)) = Mat4::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Mat4(Box::new(val)));
            } else if let Ok((range, val)) =
                Vec4UnLoop::from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                result = Some(val.into_expression());
            } else if let Ok((range, val)) = convert.meta_bool("bool") {
                convert.update(range);
                result = Some(Expression::Variable(Box::new((
                    convert.source(start).unwrap(),
                    Variable::bool(val),
                ))));
            } else if let Ok((range, val)) = convert.meta_string("color") {
                convert.update(range);
                if let Some((rgb, a)) = read_color::rgb_maybe_a(&mut val.chars()) {
                    let v = [
                        f32::from(rgb[0]) / 255.0,
                        f32::from(rgb[1]) / 255.0,
                        f32::from(rgb[2]) / 255.0,
                        f32::from(a.unwrap_or(255)) / 255.0,
                    ];
                    result = Some(Expression::Variable(Box::new((range, Variable::Vec4(v)))));
                } else {
                    return Err(());
                }
            } else if let Ok((range, val)) = Go::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Go(Box::new(val)));
            } else if let Ok((range, val)) = Call::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::Call(Box::new(val)));
            } else if let Ok((range, val)) =
                Call::named_from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Call(Box::new(val)));
            } else if let Ok((range, val)) = Assign::from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Assign(Box::new(val)));
            } else if let Ok((range, val)) = For::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::For(Box::new(val)));
            } else if let Ok((range, val)) =
                ForN::from_meta_data(file, source, "for_n", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::ForN(Box::new(val)));
            } else if let Ok((range, val)) =
                ForN::from_meta_data(file, source, "sum", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Sum(Box::new(val)));
            } else if let Ok((range, val)) =
                ForN::from_meta_data(file, source, "sum_vec4", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::SumVec4(Box::new(val)));
            } else if let Ok((range, val)) =
                ForN::from_meta_data(file, source, "prod", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Prod(Box::new(val)));
            } else if let Ok((range, val)) =
                ForN::from_meta_data(file, source, "prod_vec4", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::ProdVec4(Box::new(val)));
            } else if let Ok((range, val)) =
                ForN::from_meta_data(file, source, "min", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Min(Box::new(val)));
            } else if let Ok((range, val)) =
                ForN::from_meta_data(file, source, "max", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Max(Box::new(val)));
            } else if let Ok((range, val)) =
                ForN::from_meta_data(file, source, "sift", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Sift(Box::new(val)));
            } else if let Ok((range, val)) =
                ForN::from_meta_data(file, source, "any", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Any(Box::new(val)));
            } else if let Ok((range, val)) =
                ForN::from_meta_data(file, source, "all", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::All(Box::new(val)));
            } else if let Ok((range, val)) =
                ForN::from_meta_data(file, source, "link_for", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::LinkFor(Box::new(val)));
            } else if let Ok((range, val)) = Loop::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                result = Some(val.into_expression());
            } else if let Ok((range, val)) = If::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                result = Some(Expression::If(Box::new(val)));
            } else if let Ok((range, _)) = convert.meta_bool("try") {
                convert.update(range);
                result = Some(Expression::Try(Box::new(result.unwrap())));
            } else if let Ok((range, val)) = Swizzle::from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Swizzle(Box::new(val)));
            } else if let Ok((range, val)) =
                Closure::from_meta_data(file, source, "closure", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::Closure(Arc::new(val)));
            } else if let Ok((range, val)) =
                CallClosure::from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::CallClosure(Box::new(val)));
            } else if let Ok((range, val)) =
                CallClosure::named_from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::CallClosure(Box::new(val)));
            } else if let Ok((range, val)) = Grab::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                if let Some(v) = val.precompute() {
                    result = Some(Expression::Variable(Box::new((val.source_range, v))));
                } else {
                    result = Some(Expression::Grab(Box::new(val)));
                }
            } else if let Ok((range, val)) = TryExpr::from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::TryExpr(Box::new(val)));
            } else if let Ok((range, val)) = In::from_meta_data("in", convert, ignored) {
                convert.update(range);
                result = Some(Expression::In(Box::new(val)));
            } else if let Ok((range, val)) =
                ForIn::from_meta_data(file, source, "for_in", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::ForIn(Box::new(val)));
            } else if let Ok((range, val)) =
                ForIn::from_meta_data(file, source, "sum_in", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::SumIn(Box::new(val)));
            } else if let Ok((range, val)) =
                ForIn::from_meta_data(file, source, "prod_in", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::ProdIn(Box::new(val)));
            } else if let Ok((range, val)) =
                ForIn::from_meta_data(file, source, "min_in", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::MinIn(Box::new(val)));
            } else if let Ok((range, val)) =
                ForIn::from_meta_data(file, source, "max_in", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::MaxIn(Box::new(val)));
            } else if let Ok((range, val)) =
                ForIn::from_meta_data(file, source, "sift_in", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::SiftIn(Box::new(val)));
            } else if let Ok((range, val)) =
                ForIn::from_meta_data(file, source, "any_in", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::AnyIn(Box::new(val)));
            } else if let Ok((range, val)) =
                ForIn::from_meta_data(file, source, "all_in", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::AllIn(Box::new(val)));
            } else if let Ok((range, val)) =
                ForIn::from_meta_data(file, source, "link_in", convert, ignored)
            {
                convert.update(range);
                result = Some(Expression::LinkIn(Box::new(val)));
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
            _ => None,
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
            #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
            Go(ref go) => go.source_range,
            #[cfg(not(all(not(target_family = "wasm"), feature = "threading")))]
            Go(ref go) => match **go {},
            Call(ref call) => call.info.source_range,
            CallVoid(ref call) => call.info.source_range,
            CallReturn(ref call) => call.info.source_range,
            CallBinOp(ref call) => call.info.source_range,
            CallUnOp(ref call) => call.info.source_range,
            CallLazy(ref call) => call.info.source_range,
            CallLoaded(ref call) => call.info.source_range,
            Item(ref it) => it.source_range,
            Assign(ref assign) => assign.source_range,
            Vec4(ref vec4) => vec4.source_range,
            Mat4(ref mat4) => mat4.source_range,
            For(ref for_expr) => for_expr.source_range,
            ForN(ref for_n_expr) => for_n_expr.source_range,
            #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
            ForIn(ref for_in_expr) => for_in_expr.source_range,
            #[cfg(not(all(not(target_family = "wasm"), feature = "threading")))]
            ForIn(ref for_in_expr) |
            SumIn(ref for_in_expr) |
            ProdIn(ref for_in_expr) |
            MinIn(ref for_in_expr) |
            MaxIn(ref for_in_expr) |
            SiftIn(ref for_in_expr) |
            AnyIn(ref for_in_expr) |
            AllIn(ref for_in_expr) |
            LinkIn(ref for_in_expr) => match **for_in_expr {},
            Sum(ref for_n_expr) => for_n_expr.source_range,
            #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
            SumIn(ref for_in_expr) => for_in_expr.source_range,
            SumVec4(ref for_n_expr) => for_n_expr.source_range,
            Prod(ref for_n_expr) => for_n_expr.source_range,
            #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
            ProdIn(ref for_in_expr) => for_in_expr.source_range,
            ProdVec4(ref for_n_expr) => for_n_expr.source_range,
            Min(ref for_n_expr) => for_n_expr.source_range,
            #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
            MinIn(ref for_in_expr) => for_in_expr.source_range,
            Max(ref for_n_expr) => for_n_expr.source_range,
            #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
            MaxIn(ref for_in_expr) => for_in_expr.source_range,
            Sift(ref for_n_expr) => for_n_expr.source_range,
            #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
            SiftIn(ref for_in_expr) => for_in_expr.source_range,
            Any(ref for_n_expr) => for_n_expr.source_range,
            #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
            AnyIn(ref for_in_expr) => for_in_expr.source_range,
            All(ref for_n_expr) => for_n_expr.source_range,
            #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
            AllIn(ref for_in_expr) => for_in_expr.source_range,
            LinkFor(ref for_n_expr) => for_n_expr.source_range,
            #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
            LinkIn(ref for_in_expr) => for_in_expr.source_range,
            If(ref if_expr) => if_expr.source_range,
            Variable(ref range_var) => range_var.0,
            Try(ref expr) => expr.source_range(),
            Swizzle(ref swizzle) => swizzle.source_range,
            Closure(ref closure) => closure.source_range,
            CallClosure(ref call) => call.source_range,
            Grab(ref grab) => grab.source_range,
            TryExpr(ref try_expr) => try_expr.source_range,
            #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
            In(ref in_expr) => in_expr.source_range,
            #[cfg(not(all(not(target_family = "wasm"), feature = "threading")))]
            In(ref in_expr) => match **in_expr {},
        }
    }

    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        use self::Expression::*;

        match *self {
            Link(ref mut link) => {
                link.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            Object(ref mut obj) => {
                obj.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            Array(ref mut arr) => {
                arr.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            ArrayFill(ref mut arr_fill) => {
                arr_fill.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            Return(ref mut expr) => {
                let st = stack.len();
                expr.resolve_locals(relative, stack, closure_stack, module, use_lookup);
                stack.truncate(st);
            }
            ReturnVoid(_) => {}
            Break(_) => {}
            Continue(_) => {}
            Block(ref mut bl) => {
                bl.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            Go(ref mut go) => go.resolve_locals(relative, stack, closure_stack, module, use_lookup),
            Call(ref mut call) => {
                call.resolve_locals(relative, stack, closure_stack, module, use_lookup);
                match call.f_index {
                    FnIndex::Void(f) => {
                        *self = Expression::CallVoid(Box::new(self::CallVoid {
                            args: call.args.clone(),
                            fun: f,
                            info: call.info.clone(),
                        }))
                    }
                    FnIndex::Return(f) => {
                        *self = Expression::CallReturn(Box::new(self::CallReturn {
                            args: call.args.clone(),
                            fun: f,
                            info: call.info.clone(),
                        }))
                    }
                    FnIndex::BinOp(f) => {
                        *self = Expression::CallBinOp(Box::new(self::CallBinOp {
                            left: call.args[0].clone(),
                            right: call.args[1].clone(),
                            fun: f,
                            info: call.info.clone(),
                        }))
                    }
                    FnIndex::UnOp(f) => {
                        *self = Expression::CallUnOp(Box::new(self::CallUnOp {
                            arg: call.args[0].clone(),
                            fun: f,
                            info: call.info.clone(),
                        }))
                    }
                    FnIndex::Lazy(f, lazy_inv) => {
                        *self = Expression::CallLazy(Box::new(self::CallLazy {
                            args: call.args.clone(),
                            fun: f,
                            lazy_inv,
                            info: call.info.clone(),
                        }))
                    }
                    FnIndex::Loaded(f) => {
                        *self = Expression::CallLoaded(Box::new(self::CallLoaded {
                            args: call.args.clone(),
                            custom_source: call.custom_source.clone(),
                            fun: f,
                            info: call.info.clone(),
                        }))
                    }
                    FnIndex::None => {}
                }
            }
            CallVoid(_) => unimplemented!("`CallVoid` is transformed from `Call`"),
            CallReturn(_) => unimplemented!("`CallReturn` is transformed from `Call`"),
            CallLazy(_) => unimplemented!("`CallLazy` is transformed from `Call`"),
            CallLoaded(_) => unimplemented!("`CallLoaded` is transformed from `Call`"),
            CallBinOp(_) => unimplemented!("`CallBinOp` is transformed from `Call`"),
            CallUnOp(_) => unimplemented!("`CallUnOp` is transformed from `Call`"),
            Item(ref mut it) => {
                it.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            Assign(ref mut assign) => {
                assign.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            Vec4(ref mut vec4) => {
                vec4.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            Mat4(ref mut mat4) => {
                mat4.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            For(ref mut for_expr) => {
                for_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            ForN(ref mut for_n_expr) => {
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            ForIn(ref mut for_n_expr) |
            SumIn(ref mut for_n_expr) |
            ProdIn(ref mut for_n_expr) |
            MinIn(ref mut for_n_expr) |
            MaxIn(ref mut for_n_expr) |
            SiftIn(ref mut for_n_expr) |
            AnyIn(ref mut for_n_expr) |
            AllIn(ref mut for_n_expr) |
            LinkIn(ref mut for_n_expr) => {
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            Sum(ref mut for_n_expr) |
            SumVec4(ref mut for_n_expr) |
            Prod(ref mut for_n_expr) |
            ProdVec4(ref mut for_n_expr) |
            Min(ref mut for_n_expr) |
            Max(ref mut for_n_expr) |
            Sift(ref mut for_n_expr) |
            Any(ref mut for_n_expr) |
            All(ref mut for_n_expr) |
            LinkFor(ref mut for_n_expr) => {
                for_n_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            If(ref mut if_expr) => {
                if_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            Variable(_) => {}
            Try(ref mut expr) => {
                expr.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            Swizzle(ref mut swizzle) => {
                swizzle
                    .expr
                    .resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            Closure(ref mut closure) => Arc::make_mut(closure).resolve_locals(
                relative,
                stack,
                closure_stack,
                module,
                use_lookup,
            ),
            CallClosure(ref mut call) => {
                call.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            Grab(ref mut grab) => {
                grab.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            TryExpr(ref mut try_expr) => {
                try_expr.resolve_locals(relative, stack, closure_stack, module, use_lookup)
            }
            In(ref mut in_expr) => in_expr.resolve_locals(relative, module, use_lookup),
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Link), ()> {
        let start = convert;
        let node = "link";
        let start_range = convert.start_node(node)?;
        convert.update(start_range);

        let mut items: Vec<Expression> = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "link_item", convert, ignored)
            {
                convert.update(range);
                items.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((
            convert.subtract(start),
            Link {
                items,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn precompute(&self) -> Option<Variable> {
        let mut link = ::link::Link::new();
        for it in &self.items {
            if let Some(v) = it.precompute() {
                if link.push(&v).is_err() {
                    return None;
                };
            } else {
                return None;
            }
        }
        Some(Variable::Link(Box::new(link)))
    }

    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for expr in &mut self.items {
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Object), ()> {
        let start = convert;
        let node = "object";
        let start_range = convert.start_node(node)?;
        convert.update(start_range);

        let mut key_values = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) =
                Object::key_value_from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                key_values.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((
            convert.subtract(start),
            Object {
                key_values,
                source_range: convert.source(start).unwrap(),
            },
        ))
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, (Arc<String>, Expression)), ()> {
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "val", convert, ignored)
            {
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
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for &mut (_, ref mut expr) in &mut self.key_values {
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Array), ()> {
        let start = convert;
        let node = "array";
        let start_range = convert.start_node(node)?;
        convert.update(start_range);

        let mut items = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "array_item", convert, ignored)
            {
                convert.update(range);
                items.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((
            convert.subtract(start),
            Array {
                items,
                source_range: convert.source(start).unwrap(),
            },
        ))
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
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for item in &mut self.items {
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, ArrayFill), ()> {
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "fill", convert, ignored)
            {
                convert.update(range);
                fill = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "n", convert, ignored)
            {
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
        Ok((
            convert.subtract(start),
            ArrayFill {
                fill,
                n,
                source_range: convert.source(start).unwrap(),
            },
        ))
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
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.fill
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
        self.n
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }
}

/// Parse sequence of binary operators.
#[derive(Debug, Clone)]
pub struct BinOpSeq {
    /// Item expressions.
    pub items: Vec<Expression>,
    /// Binary operators.
    pub ops: Vec<BinOp>,
    /// The range in source.
    pub source_range: Range,
}

impl BinOpSeq {
    /// Creates multiply expression from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, BinOpSeq), ()> {
        let start = convert;
        let start_range = convert.start_node(node)?;
        convert.update(start_range);

        let mut items = vec![];
        let mut ops = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) =
                UnOpExpression::from_meta_data("neg", file, source, convert, ignored)
            {
                convert.update(range);
                items.push(val);
            } else if let Ok((range, val)) =
                BinOpSeq::from_meta_data(file, source, "pow", convert, ignored)
            {
                convert.update(range);
                items.push(val.into_expression());
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "expr", convert, ignored)
            {
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
            } else if let Ok((range, _)) = convert.meta_bool("+") {
                convert.update(range);
                ops.push(BinOp::Add);
            } else if let Ok((range, _)) = convert.meta_bool("-") {
                convert.update(range);
                ops.push(BinOp::Sub);
            } else if let Ok((range, _)) = convert.meta_bool("||") {
                convert.update(range);
                ops.push(BinOp::OrElse);
            } else if let Ok((range, _)) = convert.meta_bool("^") {
                convert.update(range);
                ops.push(BinOp::Pow);
            } else if let Ok((range, _)) = convert.meta_bool("&&") {
                convert.update(range);
                ops.push(BinOp::AndAlso);
            } else if let Ok((range, _)) = convert.meta_bool("<") {
                convert.update(range);
                ops.push(BinOp::Less);
            } else if let Ok((range, _)) = convert.meta_bool("<=") {
                convert.update(range);
                ops.push(BinOp::LessOrEqual);
            } else if let Ok((range, _)) = convert.meta_bool(">") {
                convert.update(range);
                ops.push(BinOp::Greater);
            } else if let Ok((range, _)) = convert.meta_bool(">=") {
                convert.update(range);
                ops.push(BinOp::GreaterOrEqual);
            } else if let Ok((range, _)) = convert.meta_bool("==") {
                convert.update(range);
                ops.push(BinOp::Equal);
            } else if let Ok((range, _)) = convert.meta_bool("!=") {
                convert.update(range);
                ops.push(BinOp::NotEqual);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        if items.is_empty() {
            return Err(());
        }
        Ok((
            convert.subtract(start),
            BinOpSeq {
                items,
                ops,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn into_expression(mut self) -> Expression {
        if self.items.len() == 1 {
            self.items[0].clone()
        } else {
            let op = self.ops.pop().expect("Expected a binary operation");
            let last = self.items.pop().expect("Expected argument");
            let source_range = self.source_range;
            BinOpExpression {
                op,
                left: self.into_expression(),
                right: last,
                source_range,
            }
            .into_expression()
        }
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

pub(crate) const BINOP_PREC_POW: u8 = 3;
pub(crate) const BINOP_PREC_MUL: u8 = 2;
pub(crate) const BINOP_PREC_ADD: u8 = 1;
pub(crate) const BINOP_PREC_EQ: u8 = 0;

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
            BinOp::Less => "<",
            BinOp::LessOrEqual => "<=",
            BinOp::Greater => ">",
            BinOp::GreaterOrEqual => ">=",
            BinOp::Equal => "==",
            BinOp::NotEqual => "!=",
        }
    }

    /// Returns symbol of binary operator in boolean variant.
    pub fn symbol_bool(self) -> &'static str {
        match self {
            BinOp::Add => "or",
            BinOp::Mul => "and",
            _ => self.symbol(),
        }
    }

    /// Returns the operator precedence level.
    /// Used to put parentheses in right places when printing out closures.
    pub fn precedence(self) -> u8 {
        match self {
            BinOp::Less
            | BinOp::LessOrEqual
            | BinOp::Greater
            | BinOp::GreaterOrEqual
            | BinOp::Equal
            | BinOp::NotEqual => BINOP_PREC_EQ,
            BinOp::OrElse => BINOP_PREC_ADD,
            BinOp::AndAlso => BINOP_PREC_MUL,
            BinOp::Add | BinOp::Sub => BINOP_PREC_ADD,
            BinOp::Mul | BinOp::Dot | BinOp::Cross | BinOp::Div | BinOp::Rem => BINOP_PREC_MUL,
            BinOp::Pow => BINOP_PREC_POW,
        }
    }
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
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) -> bool {
        match *self {
            Id::String(_, _) => false,
            Id::F64(_, _) => false,
            Id::Expression(ref mut expr) => {
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
            source_range,
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
            ids: self.ids.iter().take(n).cloned().collect(),
            try_ids: {
                let mut try_ids = vec![];
                for &ind in &self.try_ids {
                    if ind >= n {
                        break;
                    }
                    try_ids.push(ind);
                }
                try_ids
            },
            source_range: self.source_range,
        }
    }

    /// Creates item from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Item), ()> {
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "id", convert, ignored)
            {
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
        Ok((
            convert.subtract(start),
            Item {
                name,
                stack_id: Cell::new(None),
                static_stack_id: Cell::new(None),
                current,
                try,
                ids,
                try_ids,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn resolve_locals(
        &mut self,
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
        for id in &mut self.ids {
            if id.resolve_locals(relative, stack, closure_stack, module, use_lookup) {
                stack.push(None);
            }
        }
        stack.truncate(st);
    }
}

/// Go call.
#[cfg(all(not(target_family = "wasm"), feature = "threading"))]
#[derive(Debug, Clone)]
pub struct Go {
    /// Function call.
    pub call: Call,
    /// The range in source.
    pub source_range: Range,
}

/// Go call.
#[cfg(not(all(not(target_family = "wasm"), feature = "threading")))]
#[derive(Debug, Clone)]
pub enum Go {}

impl Go {
    #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
    /// Creates go call from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>,
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
            } else if let Ok((range, val)) = Call::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                call = Some(val);
            } else if let Ok((range, val)) =
                Call::named_from_meta_data(file, source, convert, ignored)
            {
                convert.update(range);
                call = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let call = call.ok_or(())?;
        Ok((
            convert.subtract(start),
            Go {
                call,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    #[cfg(not(all(not(target_family = "wasm"), feature = "threading")))]
    /// Creates go call from meta data.
    pub fn from_meta_data(
        _file: &Arc<String>,
        _source: &Arc<String>,
        _convert: Convert,
        _ignored: &mut Vec<Range>,
    ) -> Result<(Range, Go), ()> {
        Err(())
    }

    #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for arg in &mut self.call.args {
            let st = stack.len();
            arg.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(st);
        }
        stack.truncate(st);
    }

    #[cfg(not(all(not(target_family = "wasm"), feature = "threading")))]
    fn resolve_locals(
        &mut self,
        _relative: usize,
        _stack: &mut Vec<Option<Arc<String>>>,
        _closure_stack: &mut Vec<usize>,
        _module: &Module,
        _use_lookup: &UseLookup,
    ) {}
}

/// Call info.
#[derive(Debug, Clone)]
pub struct CallInfo {
    /// Name of function.
    pub name: Arc<String>,
    /// Alias.
    pub alias: Option<Arc<String>>,
    /// The range in source.
    pub source_range: Range,
}

/// Loaded function call.
#[derive(Debug, Clone)]
pub struct CallLoaded {
    /// Arguments.
    pub args: Vec<Expression>,
    /// Relative to function you call from.
    pub fun: isize,
    /// Info about the call.
    pub info: Box<CallInfo>,
    /// A custom source, such as when calling a function inside a loaded module.
    pub custom_source: Option<Arc<String>>,
}

/// External function call.
#[derive(Debug, Clone)]
pub struct CallLazy {
    /// Arguments.
    pub args: Vec<Expression>,
    /// Function pointer.
    pub fun: crate::FnReturnRef,
    /// Lazy invariant.
    pub lazy_inv: crate::LazyInvariant,
    /// Info about the call.
    pub info: Box<CallInfo>,
}

/// External function call.
#[derive(Debug, Clone)]
pub struct CallBinOp {
    /// Left argument.
    pub left: Expression,
    /// Right argument.
    pub right: Expression,
    /// Function pointer.
    pub fun: crate::FnBinOpRef,
    /// Info about the call.
    pub info: Box<CallInfo>,
}

/// External function call.
#[derive(Debug, Clone)]
pub struct CallUnOp {
    /// Argument.
    pub arg: Expression,
    /// Function pointer.
    pub fun: crate::FnUnOpRef,
    /// Info about the call.
    pub info: Box<CallInfo>,
}

/// External function call.
#[derive(Debug, Clone)]
pub struct CallReturn {
    /// Arguments.
    pub args: Vec<Expression>,
    /// Function pointer.
    pub fun: crate::FnReturnRef,
    /// Info about the call.
    pub info: Box<CallInfo>,
}

/// External function call.
#[derive(Debug, Clone)]
pub struct CallVoid {
    /// Arguments.
    pub args: Vec<Expression>,
    /// Function pointer.
    pub fun: crate::FnVoidRef,
    /// Info about the call.
    pub info: Box<CallInfo>,
}

/// Function call.
#[derive(Debug, Clone)]
pub struct Call {
    /// Arguments.
    pub args: Vec<Expression>,
    /// Function index.
    pub f_index: FnIndex,
    /// Info about the call.
    pub info: Box<CallInfo>,
    /// A custom source, such as when calling a function inside a loaded module.
    pub custom_source: Option<Arc<String>>,
}

impl Call {
    /// Creates call from meta data.
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Call), ()> {
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "call_arg", convert, ignored)
            {
                let mut peek = convert;
                mutable.push(match peek.start_node("call_arg") {
                    Ok(r) => {
                        peek.update(r);
                        peek.meta_bool("mut").is_ok()
                    }
                    _ => unreachable!(),
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
                if !first {
                    name_plus_args.push(',');
                }
                name_plus_args.push_str(if arg { "mut" } else { "_" });
                first = false;
            }
            name_plus_args.push(')');
            name = Arc::new(name_plus_args);
        }

        Ok((
            convert.subtract(start),
            Call {
                args,
                f_index: FnIndex::None,
                custom_source: None,
                info: Box::new(CallInfo {
                    alias,
                    name,
                    source_range: convert.source(start).unwrap(),
                }),
            },
        ))
    }

    /// Creates named argument call from meta data.
    pub fn named_from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Call), ()> {
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "call_arg", convert, ignored)
            {
                let mut peek = convert;
                mutable.push(match peek.start_node("call_arg") {
                    Ok(r) => {
                        peek.update(r);
                        peek.meta_bool("mut").is_ok()
                    }
                    _ => unreachable!(),
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
                if !first {
                    name.push(',');
                }
                name.push_str(if arg { "mut" } else { "_" });
                first = false;
            }
            name.push(')');
        }

        Ok((
            convert.subtract(start),
            Call {
                args,
                f_index: FnIndex::None,
                custom_source: None,
                info: Box::new(CallInfo {
                    alias,
                    name: Arc::new(name),
                    source_range: convert.source(start).unwrap(),
                }),
            },
        ))
    }

    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        use FnBinOpRef;
        use FnExt;
        use FnReturnRef;
        use FnUnOpRef;
        use FnVoidRef;

        let st = stack.len();
        let f_index = if let Some(ref alias) = self.info.alias {
            if let Some(&i) = use_lookup
                .aliases
                .get(alias)
                .and_then(|map| map.get(&self.info.name))
            {
                match i {
                    FnAlias::Loaded(i) => FnIndex::Loaded(i as isize - relative as isize),
                    FnAlias::External(i) => {
                        let f = &module.ext_prelude[i];
                        match f.f {
                            FnExt::Void(ff) => FnIndex::Void(FnVoidRef(ff)),
                            FnExt::Return(ff) => FnIndex::Return(FnReturnRef(ff)),
                            FnExt::BinOp(ff) => FnIndex::BinOp(FnBinOpRef(ff)),
                            FnExt::UnOp(ff) => FnIndex::UnOp(FnUnOpRef(ff)),
                        }
                    }
                }
            } else {
                FnIndex::None
            }
        } else {
            module.find_function(&self.info.name, relative)
        };
        self.f_index = f_index;
        match f_index {
            FnIndex::Loaded(f_index) => {
                let index = (f_index + relative as isize) as usize;
                if module.functions[index].returns() {
                    stack.push(None);
                }
            }
            FnIndex::Void(_)
            | FnIndex::Return(_)
            | FnIndex::Lazy(_, _)
            | FnIndex::BinOp(_)
            | FnIndex::UnOp(_) => {
                // Don't push return since last value in block
                // is used as return value.
            }
            FnIndex::None => {}
        }
        for arg in &mut self.args {
            let arg_st = stack.len();
            arg.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(arg_st);
            if let FnIndex::BinOp(_) = f_index {
            } else {
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
                _ => {
                    sum += 1;
                }
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, CallClosure), ()> {
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
            } else if let Ok((range, val)) = Item::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                item = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "call_arg", convert, ignored)
            {
                let mut peek = convert;
                mutable.push(match peek.start_node("call_arg") {
                    Ok(r) => {
                        peek.update(r);
                        peek.meta_bool("mut").is_ok()
                    }
                    _ => unreachable!(),
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
        Ok((
            convert.subtract(start),
            CallClosure {
                item,
                args,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    /// Creates named argument closure call from meta data.
    pub fn named_from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, CallClosure), ()> {
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
            } else if let Ok((range, val)) = Item::from_meta_data(file, source, convert, ignored) {
                convert.update(range);
                item = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("word") {
                convert.update(range);
                if !name.is_empty() {
                    name.push('_');
                }
                name.push_str(&val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "call_arg", convert, ignored)
            {
                let mut peek = convert;
                mutable.push(match peek.start_node("call_arg") {
                    Ok(r) => {
                        peek.update(r);
                        peek.meta_bool("mut").is_ok()
                    }
                    _ => unreachable!(),
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
        Ok((
            convert.subtract(start),
            CallClosure {
                item,
                args,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.item
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        // All closures must return a value.
        // Use return type because it has the same name.
        stack.push(Some(crate::runtime::RETURN_TYPE.clone()));
        for arg in &mut self.args {
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
                _ => {
                    sum += 1;
                }
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Norm), ()> {
        let start = convert;
        let node = "norm";
        let start_range = convert.start_node(node)?;
        convert.update(start_range);

        let mut expr: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "expr", convert, ignored)
            {
                convert.update(range);
                expr = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let expr = expr.ok_or(())?;
        Ok((
            convert.subtract(start),
            Norm {
                expr,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn into_call_expr(self) -> Expression {
        Expression::Call(Box::new(Call {
            args: vec![self.expr],
            custom_source: None,
            f_index: FnIndex::None,
            info: Box::new(CallInfo {
                alias: None,
                name: crate::NORM.clone(),
                source_range: self.source_range,
            }),
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
    fn into_expression(self) -> Expression {
        use self::BinOp::*;

        Expression::Call(Box::new(Call {
            args: vec![self.left, self.right],
            custom_source: None,
            f_index: FnIndex::None,
            info: Box::new(CallInfo {
                alias: None,
                name: match self.op {
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
                source_range: self.source_range,
            }),
        }))
    }
}

/// Unary operator expression.
pub struct UnOpExpression;

impl UnOpExpression {
    /// Creates unary operator expression from meta data.
    pub fn from_meta_data(
        node: &str,
        file: &Arc<String>,
        source: &Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Expression), ()> {
        let start = convert;
        let start_range = convert.start_node(node)?;
        convert.update(start_range);

        let mut expr: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "expr", convert, ignored)
            {
                convert.update(range);
                expr = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let expr = expr.ok_or(())?;
        Ok((convert.subtract(start), {
            let name = match node {
                "not" => crate::NOT.clone(),
                "neg" => crate::NEG.clone(),
                _ => return Err(()),
            };
            Expression::Call(Box::new(Call {
                args: vec![expr],
                custom_source: None,
                f_index: FnIndex::None,
                info: Box::new(CallInfo {
                    alias: None,
                    name,
                    source_range: convert.source(start).unwrap(),
                }),
            }))
        }))
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Assign), ()> {
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "left", convert, ignored)
            {
                convert.update(range);
                left = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "right", convert, ignored)
            {
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
        Ok((
            convert.subtract(start),
            Assign {
                op,
                left,
                right,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        // Declared locals in right expressions are popped from the stack.
        let st = stack.len();
        self.right
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);

        // Declare new local when there is an item with no extra.
        if let Expression::Item(ref item) = self.left {
            if item.ids.is_empty() && self.op == AssignOp::Assign {
                stack.push(Some(item.name.clone()));
                return;
            }
        }

        self.left
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Mat4), ()> {
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "ex", convert, ignored)
            {
                convert.update(range);
                x = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "ey", convert, ignored)
            {
                convert.update(range);
                y = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "ez", convert, ignored)
            {
                convert.update(range);
                z = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "ew", convert, ignored)
            {
                convert.update(range);
                w = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let x = x.ok_or(())?;
        let y = y.unwrap_or_else(|| {
            Expression::Variable(Box::new((
                Range::empty(0),
                Variable::Vec4([0.0, 1.0, 0.0, 0.0]),
            )))
        });
        let z = z.unwrap_or_else(|| {
            Expression::Variable(Box::new((
                Range::empty(0),
                Variable::Vec4([0.0, 0.0, 1.0, 0.0]),
            )))
        });
        let w = w.unwrap_or_else(|| {
            Expression::Variable(Box::new((
                Range::empty(0),
                Variable::Vec4([0.0, 0.0, 0.0, 1.0]),
            )))
        });
        Ok((
            convert.subtract(start),
            Mat4 {
                args: vec![x, y, z, w],
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for arg in &mut self.args {
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Vec4), ()> {
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "x", convert, ignored)
            {
                convert.update(range);
                x = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "y", convert, ignored)
            {
                convert.update(range);
                y = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "z", convert, ignored)
            {
                convert.update(range);
                z = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "w", convert, ignored)
            {
                convert.update(range);
                w = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let x = x.ok_or(())?;
        let y = y.unwrap_or_else(|| {
            Expression::Variable(Box::new((Range::empty(0), Variable::f64(0.0))))
        });
        let z = z.unwrap_or_else(|| {
            Expression::Variable(Box::new((Range::empty(0), Variable::f64(0.0))))
        });
        let w = w.unwrap_or_else(|| {
            Expression::Variable(Box::new((Range::empty(0), Variable::f64(0.0))))
        });
        Ok((
            convert.subtract(start),
            Vec4 {
                args: vec![x, y, z, w],
                source_range: convert.source(start).unwrap(),
            },
        ))
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
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        for arg in &mut self.args {
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Vec4UnLoop), ()> {
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "expr", convert, ignored)
            {
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
        Ok((
            convert.subtract(start),
            Vec4UnLoop {
                name,
                expr,
                len,
                source_range: convert.source(start).unwrap(),
            },
        ))
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Swizzle), ()> {
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
            } else if let Ok((range, val)) = Sw::from_meta_data("sw0", convert, ignored) {
                convert.update(range);
                sw0 = Some(val.ind);
            } else if let Ok((range, val)) = Sw::from_meta_data("sw1", convert, ignored) {
                convert.update(range);
                sw1 = Some(val.ind);
            } else if let Ok((range, val)) = Sw::from_meta_data("sw2", convert, ignored) {
                convert.update(range);
                sw2 = Some(val.ind);
            } else if let Ok((range, val)) = Sw::from_meta_data("sw3", convert, ignored) {
                convert.update(range);
                sw3 = Some(val.ind);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "expr", convert, ignored)
            {
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
        Ok((
            convert.subtract(start),
            Swizzle {
                sw0,
                sw1,
                sw2,
                sw3,
                expr,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn len(&self) -> usize {
        2 + if self.sw2.is_some() { 1 } else { 0 } + if self.sw3.is_some() { 1 } else { 0 }
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Sw), ()> {
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
        Ok((
            convert.subtract(start),
            Sw {
                ind,
                source_range: convert.source(start).unwrap(),
            },
        ))
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, For), ()> {
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "init", convert, ignored)
            {
                convert.update(range);
                init = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "cond", convert, ignored)
            {
                convert.update(range);
                cond = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "step", convert, ignored)
            {
                convert.update(range);
                step = Some(val);
            } else if let Ok((range, val)) =
                Block::from_meta_data(file, source, "block", convert, ignored)
            {
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
        Ok((
            convert.subtract(start),
            For {
                init,
                cond,
                step,
                block,
                label,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.init
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        let after_init = stack.len();
        self.cond
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(after_init);
        self.step
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(after_init);
        self.block
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }
}

/// For-In expression.
#[derive(Debug, Clone)]
#[cfg(all(not(target_family = "wasm"), feature = "threading"))]
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

/// For-In expression.
#[derive(Debug, Clone)]
#[cfg(not(all(not(target_family = "wasm"), feature = "threading")))]
pub enum ForIn {}

impl ForIn {
    /// Creates For-In expression from meta data.
    #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
    pub fn from_meta_data(
        file: &Arc<String>,
        source: &Arc<String>,
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, ForIn), ()> {
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "iter", convert, ignored)
            {
                convert.update(range);
                iter = Some(val);
            } else if let Ok((range, val)) =
                Block::from_meta_data(file, source, "block", convert, ignored)
            {
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
        Ok((
            convert.subtract(start),
            ForIn {
                name,
                iter,
                block,
                label,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    /// Creates For-In expression from meta data.
    #[cfg(not(all(not(target_family = "wasm"), feature = "threading")))]
    pub fn from_meta_data(
        _file: &Arc<String>,
        _source: &Arc<String>,
        _node: &str,
        _convert: Convert,
        _ignored: &mut Vec<Range>,
    ) -> Result<(Range, ForIn), ()> {
        Err(())
    }

    #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.iter
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
        stack.push(Some(self.name.clone()));
        self.block
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
    }

    #[cfg(not(all(not(target_family = "wasm"), feature = "threading")))]
    fn resolve_locals(
        &mut self,
        _relative: usize,
        _stack: &mut Vec<Option<Arc<String>>>,
        _closure_stack: &mut Vec<usize>,
        _module: &Module,
        _use_lookup: &UseLookup,
    ) {}
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, ForN), ()> {
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
            } else if let Ok((range, val)) =
                Block::from_meta_data(file, source, "block", convert, ignored)
            {
                convert.update(range);
                block = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("label") {
                convert.update(range);
                label = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                let mut start_expr: Option<Expression> = None;
                let mut end_expr: Option<Expression> = None;
                if let Ok((range, val)) =
                    Expression::from_meta_data(file, source, "start", convert, ignored)
                {
                    convert.update(range);
                    start_expr = Some(val);
                }
                if let Ok((range, val)) =
                    Expression::from_meta_data(file, source, "end", convert, ignored)
                {
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
            block,
        )
    }

    fn create(
        node: &str,
        range: Range,
        source_range: Range,
        label: Option<Arc<String>>,
        indices: &[(Arc<String>, Option<Expression>, Option<Expression>)],
        mut block: Option<Block>,
    ) -> Result<(Range, ForN), ()> {
        if indices.is_empty() {
            return Err(());
        }

        let name: Arc<String> = indices[0].0.clone();
        let start_expr = indices[0].1.clone();
        let mut end_expr = indices[0].2.clone();

        if indices.len() > 1 {
            let (_, new_for_n) =
                ForN::create(node, range, source_range, None, &indices[1..], block)?;
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
                    _ => return Err(()),
                }],
            });
        }

        let block = block.ok_or(())?;

        // Infer list length from index.
        if end_expr.is_none() {
            end_expr = infer_len::infer(&block, &name);
        }

        let end_expr = end_expr.ok_or(())?;
        Ok((
            range,
            ForN {
                name,
                start: start_expr,
                end: end_expr,
                block,
                label,
                source_range,
            },
        ))
    }

    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        if let Some(ref mut start) = self.start {
            start.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(st);
        }
        self.end
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
        stack.push(Some(self.name.clone()));
        self.block
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Loop), ()> {
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
            } else if let Ok((range, val)) =
                Block::from_meta_data(file, source, "block", convert, ignored)
            {
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
        Ok((
            convert.subtract(start),
            Loop {
                block,
                label,
                source_range: convert.source(start).unwrap(),
            },
        ))
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Break), ()> {
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

        Ok((
            convert.subtract(start),
            Break {
                label,
                source_range: convert.source(start).unwrap(),
            },
        ))
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, Continue), ()> {
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

        Ok((
            convert.subtract(start),
            Continue {
                label,
                source_range: convert.source(start).unwrap(),
            },
        ))
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
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, If), ()> {
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
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "cond", convert, ignored)
            {
                convert.update(range);
                cond = Some(val);
            } else if let Ok((range, val)) =
                Block::from_meta_data(file, source, "true_block", convert, ignored)
            {
                convert.update(range);
                true_block = Some(val);
            } else if let Ok((range, val)) =
                Expression::from_meta_data(file, source, "else_if_cond", convert, ignored)
            {
                convert.update(range);
                else_if_conds.push(val);
            } else if let Ok((range, val)) =
                Block::from_meta_data(file, source, "else_if_block", convert, ignored)
            {
                convert.update(range);
                else_if_blocks.push(val);
            } else if let Ok((range, val)) =
                Block::from_meta_data(file, source, "else_block", convert, ignored)
            {
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
        Ok((
            convert.subtract(start),
            If {
                cond,
                true_block,
                else_if_conds,
                else_if_blocks,
                else_block,
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    fn resolve_locals(
        &mut self,
        relative: usize,
        stack: &mut Vec<Option<Arc<String>>>,
        closure_stack: &mut Vec<usize>,
        module: &Module,
        use_lookup: &UseLookup,
    ) {
        let st = stack.len();
        self.cond
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
        self.true_block
            .resolve_locals(relative, stack, closure_stack, module, use_lookup);
        stack.truncate(st);
        // Does not matter that conditions are resolved before blocks,
        // since the stack gets truncated anyway.
        for else_if_cond in &mut self.else_if_conds {
            else_if_cond.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(st);
        }
        for else_if_block in &mut self.else_if_blocks {
            else_if_block.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(st);
        }
        if let Some(ref mut else_block) = self.else_block {
            else_block.resolve_locals(relative, stack, closure_stack, module, use_lookup);
            stack.truncate(st);
        }
    }
}

/// Stores `in <function>` expression.
#[derive(Debug, Clone)]
#[cfg(all(not(target_family = "wasm"), feature = "threading"))]
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

/// Stores `in <function>` expression.
#[derive(Debug, Clone)]
#[cfg(not(all(not(target_family = "wasm"), feature = "threading")))]
pub enum In {}

impl In {
    /// Creates in expression from meta data.
    #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
    pub fn from_meta_data(
        node: &'static str,
        mut convert: Convert,
        ignored: &mut Vec<Range>,
    ) -> Result<(Range, In), ()> {
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
        Ok((
            convert.subtract(start),
            In {
                alias,
                name,
                f_index: Cell::new(FnIndex::None),
                source_range: convert.source(start).unwrap(),
            },
        ))
    }

    /// Creates in expression from meta data.
    #[cfg(not(all(not(target_family = "wasm"), feature = "threading")))]
    pub fn from_meta_data(
        _node: &'static str,
        _convert: Convert,
        _ignored: &mut Vec<Range>,
    ) -> Result<(Range, In), ()> {
        Err(())
    }

    #[cfg(all(not(target_family = "wasm"), feature = "threading"))]
    fn resolve_locals(&mut self, relative: usize, module: &Module, use_lookup: &UseLookup) {
        use FnBinOpRef;
        use FnExt;
        use FnReturnRef;
        use FnUnOpRef;
        use FnVoidRef;

        let f_index = if let Some(ref alias) = self.alias {
            if let Some(&i) = use_lookup
                .aliases
                .get(alias)
                .and_then(|map| map.get(&self.name))
            {
                match i {
                    FnAlias::Loaded(i) => FnIndex::Loaded(i as isize - relative as isize),
                    FnAlias::External(i) => {
                        let f = &module.ext_prelude[i];
                        match f.f {
                            FnExt::Void(ff) => FnIndex::Void(FnVoidRef(ff)),
                            FnExt::Return(ff) => FnIndex::Return(FnReturnRef(ff)),
                            FnExt::BinOp(ff) => FnIndex::BinOp(FnBinOpRef(ff)),
                            FnExt::UnOp(ff) => FnIndex::UnOp(FnUnOpRef(ff)),
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

    #[cfg(not(all(not(target_family = "wasm"), feature = "threading")))]
    fn resolve_locals(&mut self, _relative: usize, _module: &Module, _use_lookup: &UseLookup) {}
}
