use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    ast,
    Lazy,
    Module,
    Type,
};

/// Argument lifetime constraint.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Lt {
    /// Outlives another argument.
    Arg(usize),
    /// Outlives the return value on the stack.
    ///
    /// This means that some variable must be declared
    /// and referenced before calling.
    Return,
    /// No specified lifetime.
    ///
    /// This means that the argument might be created
    /// after the return value on the stack.
    Default,
}

/// Stores preloaded function constraints.
/// These are already checked.
#[derive(Clone, PartialEq, Debug)]
pub struct Dfn {
    /// Lifetimes of argument.
    pub lts: Vec<Lt>,
    /// Argument types of function.
    pub tys: Vec<Type>,
    /// Return type of function.
    pub ret: Type,
    /// Extra type information.
    ///
    /// Stores type variables, argument types, return type.
    pub ext: Vec<(Vec<Arc<String>>, Vec<Type>, Type)>,
    /// Stores lazy invariants.
    pub lazy: &'static [&'static [Lazy]],
}

impl Dfn {
    /// Creates a new function signature with no lifetime or refinement.
    pub fn nl(args: Vec<Type>, ret: Type) -> Dfn {
        Dfn {
            lts: vec![Lt::Default; args.len()],
            tys: args,
            ret,
            ext: vec![],
            lazy: crate::LAZY_NO,
        }
    }

    /// Creates new function signature from an AST function.
    pub fn new(f: &ast::Function) -> Dfn {
        let mut lts: Vec<Lt> = vec![];
        let mut tys: Vec<Type> = vec![];
        'next_arg: for arg in &f.args {
            tys.push(arg.ty.clone());
            if let Some(ref lt) = arg.lifetime {
                if **lt == "return" {
                    lts.push(Lt::Return);
                    continue 'next_arg;
                }
                for (i, arg2) in f.args.iter().enumerate() {
                    if **arg2.name == **lt {
                        lts.push(Lt::Arg(i));
                        continue 'next_arg;
                    }
                }
                panic!("Could not find argument `{}`", lt);
            } else {
                lts.push(Lt::Default);
            }
        }
        Dfn {
            lts,
            tys,
            ret: f.ret.clone(),
            ext: vec![],
            lazy: crate::LAZY_NO,
        }
    }

    /// Returns `true` if the function returns something.
    pub fn returns(&self) -> bool {
        self.ret != Type::Void
    }
}

/// Stores a prelude, used to load standard intrinsics and type check new modules.
pub struct Prelude {
    pub(crate) functions: HashMap<Arc<String>, usize>,
    pub(crate) list: Vec<Dfn>,
    pub(crate) namespaces: Vec<(Arc<Vec<Arc<String>>>, Arc<String>)>,
}

impl Default for Prelude {
    fn default() -> Prelude {
        Prelude::new()
    }
}

impl Prelude {
    /// Adds type information of function.
    pub fn insert(&mut self, namespace: Arc<Vec<Arc<String>>>, name: Arc<String>, f: Dfn) {
        let n = self.list.len();
        self.functions.insert(name.clone(), n);
        self.list.push(f);
        self.namespaces.push((namespace, name));
    }

    /// Creates a new prelude.
    pub fn new() -> Prelude {
        Prelude {
            functions: HashMap::new(),
            list: vec![],
            namespaces: vec![],
        }
    }

    /// Creates prelude from existing module.
    pub fn from_module(module: &Module) -> Prelude {
        let mut prelude = Prelude::new();
        for f in &*module.ext_prelude {
            prelude.insert(f.namespace.clone(), f.name.clone(), f.p.clone());
        }
        for f in &module.functions {
            prelude.insert(f.namespace.clone(), f.name.clone(), Dfn::new(f));
        }
        prelude
    }
}
