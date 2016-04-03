use std::sync::Arc;
use std::collections::HashMap;

use ast;
use intrinsics;
use Module;

/// Argument lifetime constraint.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Lt {
    Arg(usize),
    Return,
    Default,
}

/// Stores preloaded function constraints.
/// These are already checked.
#[derive(Clone)]
pub struct PreludeFunction {
    pub lts: Vec<Lt>,
    pub returns: bool,
}

impl PreludeFunction {
    pub fn new(f: &ast::Function) -> PreludeFunction {
        let mut lts: Vec<Lt> = vec![];
        'next_arg: for arg in &f.args {
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
        PreludeFunction {
            lts: lts,
            returns: f.returns,
        }
    }
}

pub struct Prelude {
    pub functions: HashMap<Arc<String>, PreludeFunction>
}

impl Prelude {
    pub fn from_module(module: &Module) -> Prelude {
        let mut functions = HashMap::new();
        intrinsics::standard(&mut functions);
        for (key, &(_, ref val)) in &module.ext_prelude {
            functions.insert(key.clone(), val.clone());
        }
        for f in module.functions.values() {
            functions.insert(f.name.clone(), PreludeFunction::new(f));
        }
        Prelude {
            functions: functions
        }
    }
}
