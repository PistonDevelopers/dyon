use std::sync::Arc;
use std::collections::HashMap;

use Lt;
use Module;
use Variable;

/// Lists all functions available in a module.
pub fn list_functions(module: &Module) -> Vec<Variable> {
    let mut functions = vec![];
    let name: Arc<String> = Arc::new("name".into());
    let arguments: Arc<String> = Arc::new("arguments".into());
    let returns: Arc<String> = Arc::new("returns".into());
    let takes: Arc<String> = Arc::new("takes".into());
    let lifetime: Arc<String> = Arc::new("lifetime".into());
    let ret_lifetime: Arc<String> = Arc::new("return".into());
    let ty: Arc<String> = Arc::new("type".into());
    let external: Arc<String> = Arc::new("external".into());
    let loaded: Arc<String> = Arc::new("loaded".into());
    for f in &*module.ext_prelude {
        let mut obj = HashMap::new();
        obj.insert(name.clone(), Variable::Text(f.name.clone()));
        obj.insert(returns.clone(), Variable::Text(Arc::new(f.p.ret.description())));
        obj.insert(ty.clone(), Variable::Text(external.clone()));
        let mut args = vec![];
        for (i, lt) in f.p.lts.iter().enumerate() {
            let mut obj_arg = HashMap::new();
            obj_arg.insert(name.clone(),
                Variable::Text(Arc::new(format!("arg{}", i))));
            obj_arg.insert(lifetime.clone(), match *lt {
                Lt::Default => Variable::Option(None),
                Lt::Arg(ind) => Variable::Option(Some(
                        Box::new(Variable::Text(
                            Arc::new(format!("arg{}", ind))
                        ))
                    )),
                Lt::Return => Variable::Option(Some(
                        Box::new(Variable::Text(ret_lifetime.clone()))
                    )),
            });
            obj_arg.insert(takes.clone(),
                Variable::Text(Arc::new(f.p.tys[i].description())));
            args.push(Variable::Object(Arc::new(obj_arg)));
        }
        obj.insert(arguments.clone(), Variable::Array(Arc::new(args)));
        functions.push(Variable::Object(Arc::new(obj)));
    }
    for f in &module.functions {
        let mut obj = HashMap::new();
        obj.insert(name.clone(), Variable::Text(f.name.clone()));
        obj.insert(returns.clone(), Variable::Text(Arc::new(f.ret.description())));
        obj.insert(ty.clone(), Variable::Text(loaded.clone()));
        let mut args = vec![];
        for arg in &f.args {
            let mut obj_arg = HashMap::new();
            obj_arg.insert(name.clone(),
                Variable::Text(arg.name.clone()));
            obj_arg.insert(lifetime.clone(),
                match arg.lifetime {
                    None => Variable::Option(None),
                    Some(ref lt) => Variable::Option(Some(Box::new(
                            Variable::Text(lt.clone())
                        )))
                }
            );
            obj_arg.insert(takes.clone(),
                Variable::Text(Arc::new(arg.ty.description())));
            args.push(Variable::Object(Arc::new(obj_arg)));
        }
        obj.insert(arguments.clone(), Variable::Array(Arc::new(args)));
        functions.push(Variable::Object(Arc::new(obj)));
    }
    // Sort by function names.
    functions.sort_by(|a, b|
        match (a, b) {
            (&Variable::Object(ref a), &Variable::Object(ref b)) => {
                match (&a[&name], &b[&name]) {
                    (&Variable::Text(ref a), &Variable::Text(ref b)) => {
                        a.cmp(b)
                    }
                    _ => panic!("Expected two strings")
                }
            }
            _ => panic!("Expected two objects")
        }
    );
    functions
}
