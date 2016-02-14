extern crate rand;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use self::rand::Rng;
use ast;

use Variable;
use Array;
use Object;
use Module;

/// Which side an expression is evalutated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// Whether to insert key in object when missing.
    LeftInsert(bool),
    Right
}

// TODO: Find precise semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expect {
    Nothing,
    Something
}

pub enum Flow {
    /// Continues execution.
    Continue,
    /// Return from function.
    Return,
    /// Break loop, with optional label.
    Break(Option<Arc<String>>),
    /// Continue loop, with optional label.
    ContinueLoop(Option<Arc<String>>),
}

pub struct Runtime {
    pub stack: Vec<Variable>,
    /// name, stack_len, local_len, returns.
    pub call_stack: Vec<(Arc<String>, usize, usize)>,
    pub local_stack: Vec<(Arc<String>, usize)>,
    pub ret: Arc<String>,
    pub rng: rand::ThreadRng,
    pub text_type: Variable,
    pub f64_type: Variable,
    pub return_type: Variable,
    pub bool_type: Variable,
    pub object_type: Variable,
    pub array_type: Variable,
    pub ref_type: Variable,
    pub unsafe_ref_type: Variable,
    pub rust_object_type: Variable,
}

fn resolve<'a>(stack: &'a Vec<Variable>, var: &'a Variable) -> &'a Variable {
    match *var {
        Variable::Ref(ind) => &stack[ind],
        _ => var
    }
}

fn deep_clone(v: &Variable, stack: &Vec<Variable>) -> Variable {
    use Variable::*;

    match *v {
        F64(_) => v.clone(),
        Return => v.clone(),
        Bool(_) => v.clone(),
        Text(_) => v.clone(),
        Object(ref obj) => {
            let mut res = obj.clone();
            for (_, val) in &mut res {
                *val = deep_clone(val, stack);
            }
            Object(res)
        }
        Array(ref arr) => {
            let mut res = arr.clone();
            for it in &mut res {
                *it = deep_clone(it, stack);
            }
            Array(res)
        }
        Ref(ind) => {
            deep_clone(&stack[ind], stack)
        }
        UnsafeRef(_) => panic!("Unsafe reference can not be cloned"),
        RustObject(_) => v.clone()
    }
}

// Looks up an item from a variable property.
fn item_lookup(
    var: *mut Variable,
    stack: &mut [Variable],
    prop: &ast::Id,
    start_stack_len: usize,
    expr_j: &mut usize,
    insert: bool, // Whether to insert key in object.
    last: bool,   // Whether it is the last property.
) -> *mut Variable {
    use ast::Id;
    use std::collections::hash_map::Entry;

    unsafe {
        match *var {
            Variable::Object(ref mut obj) => {
                let id = match prop {
                    &Id::String(ref id) => id.clone(),
                    &Id::Expression(_) => {
                        let id = start_stack_len + *expr_j;
                        // Resolve reference of computed expression.
                        let id = if let &Variable::Ref(ref_id) = &stack[id] {
                                ref_id
                            } else {
                                id
                            };
                        match &mut stack[id] {
                            &mut Variable::Text(ref id) => {
                                *expr_j += 1;
                                id.clone()
                            }
                            _ => panic!("Expected string")
                        }
                    }
                    _ => panic!("Expected object")
                };
                let v = match obj.entry(id.clone()) {
                    Entry::Vacant(vac) => {
                        if insert && last {
                            // Insert a key to overwrite with new value.
                            vac.insert(Variable::Return)
                        } else {
                            panic!("Object has no key `{}`", id);
                        }
                    }
                    Entry::Occupied(v) => v.into_mut()
                };
                // Resolve reference.
                if let &mut Variable::Ref(id) = v {
                    // Do not resolve if last, because references should be
                    // copy-on-write.
                    if last {
                        v
                    } else {
                        &mut stack[id]
                    }
                } else {
                    v
                }
            }
            Variable::Array(ref mut arr) => {
                let id = match prop {
                    &Id::F64(id) => id,
                    &Id::Expression(_) => {
                        let id = start_stack_len + *expr_j;
                        // Resolve reference of computed expression.
                        let id = if let &Variable::Ref(ref_id) = &stack[id] {
                                ref_id
                            } else {
                                id
                            };
                        match &mut stack[id] {
                            &mut Variable::F64(id) => {
                                *expr_j += 1;
                                id
                            }
                            _ => panic!("Expected number")
                        }
                    }
                    _ => panic!("Expected array")
                };
                let v = &mut arr[id as usize];
                // Resolve reference.
                if let &mut Variable::Ref(id) = v {
                    // Do not resolve if last, because references should be
                    // copy-on-write.
                    if last {
                        v
                    } else {
                        &mut stack[id]
                    }
                } else {
                    v
                }
            }
            _ => panic!("Expected object or array")
        }
    }
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            stack: vec![],
            call_stack: vec![],
            local_stack: vec![],
            ret: Arc::new("return".into()),
            rng: rand::thread_rng(),
            text_type: Variable::Text(Arc::new("string".into())),
            f64_type: Variable::Text(Arc::new("number".into())),
            return_type: Variable::Text(Arc::new("return".into())),
            bool_type: Variable::Text(Arc::new("boolean".into())),
            object_type: Variable::Text(Arc::new("object".into())),
            array_type: Variable::Text(Arc::new("array".into())),
            ref_type: Variable::Text(Arc::new("ref".into())),
            unsafe_ref_type: Variable::Text(Arc::new("unsafe_ref".into())),
            rust_object_type: Variable::Text(Arc::new("rust_object".into())),
        }
    }

    fn resolve<'a>(&'a self, var: &'a Variable) -> &'a Variable {
        resolve(&self.stack, var)
    }

    fn print_variable(&self, v: &Variable) {
        match *self.resolve(v) {
            Variable::Text(ref t) => {
                print!("{}", t);
            }
            Variable::F64(x) => {
                print!("{}", x);
            }
            Variable::Bool(x) => {
                print!("{}", x);
            }
            Variable::Ref(ind) => {
                self.print_variable(&self.stack[ind]);
            }
            Variable::Object(ref obj) => {
                print!("{{");
                let n = obj.len();
                for (i, (k, v)) in obj.iter().enumerate() {
                    print!("{}: ", k);
                    self.print_variable(v);
                    if i + 1 < n {
                        print!(", ");
                    }
                }
                print!("}}");
            }
            Variable::Array(ref arr) => {
                print!("[");
                let n = arr.len();
                for (i, v) in arr.iter().enumerate() {
                    self.print_variable(v);
                    if i + 1 < n {
                        print!(", ");
                    }
                }
                print!("]");
            }
            ref x => panic!("Could not print out `{:?}`", x)
        }
    }

    fn unary_f64<F: FnOnce(f64) -> f64>(&mut self, f: F) -> Expect {
        let x = self.stack.pop().expect("There is no value on the stack");
        match self.resolve(&x) {
            &Variable::F64(a) => {
                self.stack.push(Variable::F64(f(a)));
            }
            _ => panic!("Expected number")
        }
        Expect::Something
    }

    fn push_fn(&mut self, name: Arc<String>, st: usize, lc: usize) {
        self.call_stack.push((
            name,
            st,
            lc
        ));
    }
    fn pop_fn(&mut self, name: Arc<String>) {
        match self.call_stack.pop() {
            None => panic!("Did not call `{}`", name),
            Some((fn_name, st, lc)) => {
                if name != fn_name {
                    panic!("Calling `{}`, did not call `{}`", fn_name, name);
                }
                self.stack.truncate(st);
                self.local_stack.truncate(lc);
            }
        }
    }

    fn expression(
        &mut self,
        expr: &ast::Expression,
        side: Side,
        module: &Module
    ) -> (Expect, Flow) {
        use ast::Expression::*;

        match *expr {
            Object(ref obj) => {
                self.object(obj, module);
                (Expect::Something, Flow::Continue)
            }
            Array(ref arr) => {
                self.array(arr, module);
                (Expect::Something, Flow::Continue)
            }
            Block(ref block) => self.block(block, module),
            Return(ref ret) => {
                use ast::{AssignOp, Expression, Item};

                // Assign return value and then break the flow.
                let item = Expression::Item(Item {
                        name: self.ret.clone(),
                        ids: vec![]
                    });
                self.assign_specific(AssignOp::Set, &item, ret, module);
                (Expect::Something, Flow::Return)
            }
            ReturnVoid => {
                (Expect::Nothing, Flow::Return)
            }
            Break(ref b) => (Expect::Nothing, Flow::Break(b.label.clone())),
            Continue(ref b) => (Expect::Nothing, Flow::ContinueLoop(b.label.clone())),
            Call(ref call) => self.call(call, module),
            Item(ref item) => {
                self.item(item, side, module);
                (Expect::Something, Flow::Continue)
            }
            UnOp(ref unop) => (Expect::Something,
                               self.unop(unop, side, module)),
            BinOp(ref binop) => (Expect::Something,
                                 self.binop(binop, side, module)),
            Assign(ref assign) => (Expect::Nothing,
                                   self.assign(assign, module)),
            Number(ref num) => {
                self.number(num);
                (Expect::Something, Flow::Continue)
            }
            Text(ref text) => {
                self.text(text);
                (Expect::Something, Flow::Continue)
            }
            Bool(ref b) => {
                self.bool(b);
                (Expect::Something, Flow::Continue)
            }
            For(ref for_expr) => (Expect::Nothing,
                                  self.for_expr(for_expr, module)),
            If(ref if_expr) => self.if_expr(if_expr, module),
            Compare(ref compare) => (Expect::Something,
                                     self.compare(compare, module)),
            Variable(ref var) => {
                self.stack.push(var.clone());
                (Expect::Something, Flow::Continue)
            }
        }
    }

    pub fn run(&mut self, module: &Module) {
        let call = ast::Call {
            name: Arc::new("main".into()),
            args: vec![]
        };
        match module.functions.get(&call.name) {
            Some(f) => {
                if f.args.len() != 0 {
                    panic!("`main` should not have arguments");
                }
                self.call(&call, &module);
            }
            None => panic!("Could not find function `main`")
        }
    }

    fn block(&mut self, block: &ast::Block, module: &Module) -> (Expect, Flow) {
        let mut expect = Expect::Nothing;
        let lc = self.local_stack.len();
        for e in &block.expressions {
            expect = match self.expression(e, Side::Right, module) {
                (x, Flow::Continue) => x,
                x => { return x; }
            }
        }
        self.local_stack.truncate(lc);
        (expect, Flow::Continue)
    }

    fn call(&mut self, call: &ast::Call, module: &Module) -> (Expect, Flow) {
        match module.functions.get(&call.name) {
            None => {
                let st = self.stack.len();
                let lc = self.local_stack.len();
                for arg in &call.args {
                    match self.expression(arg, Side::Right, module) {
                        (x, Flow::Return) => { return (x, Flow::Return); }
                        (Expect::Something, Flow::Continue) => {}
                        _ => panic!("Expected something from argument")
                    };
                }
                let expect = match &**call.name {
                    "clone" => {
                        self.push_fn(call.name.clone(), st + 1, lc);
                        let v = self.stack.pop()
                            .expect("There is no value on the stack");
                        let v = deep_clone(self.resolve(&v), &self.stack);
                        self.stack.push(v);
                        self.pop_fn(call.name.clone());
                        Expect::Something
                    }
                    "println" => {
                        self.push_fn(call.name.clone(), st, lc);
                        let x = self.stack.pop()
                            .expect("There is no value on the stack");
                        self.print_variable(&x);
                        println!("");
                        self.pop_fn(call.name.clone());
                        Expect::Nothing
                    }
                    "print" => {
                        self.push_fn(call.name.clone(), st, lc);
                        let x = self.stack.pop()
                            .expect("There is no value on the stack");
                        self.print_variable(&x);
                        self.pop_fn(call.name.clone());
                        Expect::Nothing
                    }
                    "sqrt" => self.unary_f64(|a| a.sqrt()),
                    "sin" => self.unary_f64(|a| a.sin()),
                    "asin" => self.unary_f64(|a| a.asin()),
                    "cos" => self.unary_f64(|a| a.cos()),
                    "acos" => self.unary_f64(|a| a.acos()),
                    "tan" => self.unary_f64(|a| a.tan()),
                    "atan" => self.unary_f64(|a| a.atan()),
                    "exp" => self.unary_f64(|a| a.exp()),
                    "ln" => self.unary_f64(|a| a.ln()),
                    "log2" => self.unary_f64(|a| a.log2()),
                    "log10" => self.unary_f64(|a| a.log10()),
                    "sleep" => {
                        use std::thread::sleep;
                        use std::time::Duration;

                        self.push_fn(call.name.clone(), st, lc);
                        let v = match self.stack.pop() {
                            Some(Variable::F64(b)) => b,
                            Some(_) => panic!("Expected number"),
                            None => panic!("There is no value on the stack")
                        };
                        let secs = v as u64;
                        let nanos = (v.fract() * 1.0e9) as u32;
                        sleep(Duration::new(secs, nanos));
                        self.pop_fn(call.name.clone());
                        Expect::Nothing
                    }
                    "random" => {
                        self.push_fn(call.name.clone(), st + 1, lc);
                        let v = Variable::F64(self.rng.gen());
                        self.stack.push(v);
                        self.pop_fn(call.name.clone());
                        Expect::Something
                    }
                    "round" => {
                        self.push_fn(call.name.clone(), st + 1, lc);
                        let v = match self.stack.pop() {
                            Some(Variable::F64(b)) => b,
                            Some(_) => panic!("Expected number"),
                            None => panic!("There is no value on the stack")
                        };
                        let v = Variable::F64(v.round());
                        self.stack.push(v);
                        self.pop_fn(call.name.clone());
                        Expect::Something
                    }
                    "len" => {
                        self.push_fn(call.name.clone(), st + 1, lc);
                        let v = match self.stack.pop() {
                            Some(v) => v,
                            None => panic!("There is no value on the stack")
                        };

                        let v = {
                            let arr = match self.resolve(&v) {
                                &Variable::Array(ref arr) => arr,
                                _ => panic!("Expected array")
                            };
                            Variable::F64(arr.len() as f64)
                        };
                        self.stack.push(v);
                        self.pop_fn(call.name.clone());
                        Expect::Something
                    }
                    "push" => {
                        self.push_fn(call.name.clone(), st + 1, lc);
                        let item = match self.stack.pop() {
                            Some(item) => item,
                            None => panic!("There is no value on the stack")
                        };
                        let v = match self.stack.pop() {
                            Some(v) => v,
                            None => panic!("There is no value on the stack")
                        };

                        if let Variable::Ref(ind) = v {
                            if let Variable::Array(ref mut arr) =
                            self.stack[ind] {
                                arr.push(item);
                            } else {
                                panic!("Expected reference to array");
                            }
                        } else {
                            panic!("Expected reference to array");
                        }
                        self.pop_fn(call.name.clone());
                        Expect::Nothing
                    }
                    "read_line" => {
                        use std::io::{self, Write};

                        self.push_fn(call.name.clone(), st + 1, lc);
                        let mut input = String::new();
                        io::stdout().flush().unwrap();
                        match io::stdin().read_line(&mut input) {
                            Ok(_) => {}
                            Err(error) => panic!("{}", error)
                        };
                        self.stack.push(Variable::Text(Arc::new(input)));
                        self.pop_fn(call.name.clone());
                        Expect::Something
                    }
                    "read_number" => {
                        use std::io::{self, Write};

                        self.push_fn(call.name.clone(), st + 1, lc);
                        let err = match self.stack.pop() {
                            Some(Variable::Text(t)) => t,
                            Some(_) => panic!("Expected text"),
                            None => panic!("There is no value on the stack")
                        };
                        let stdin = io::stdin();
                        let mut stdout = io::stdout();
                        let mut input = String::new();
                        loop {
                            stdout.flush().unwrap();
                            match stdin.read_line(&mut input) {
                                Ok(_) => {}
                                Err(error) => panic!("{}", error)
                            };
                            match input.trim().parse::<f64>() {
                                Ok(v) => {
                                    self.stack.push(Variable::F64(v));
                                    break;
                                }
                                Err(_) => {
                                    println!("{}", err);
                                }
                            }
                        }
                        self.pop_fn(call.name.clone());
                        Expect::Something
                    }
                    "trim_right" => {
                        self.push_fn(call.name.clone(), st + 1, lc);
                        let mut v = match self.stack.pop() {
                            Some(Variable::Text(t)) => t,
                            Some(_) => panic!("Expected text"),
                            None => panic!("There is no value on the stack")
                        };
                        {
                            let w = Arc::make_mut(&mut v);
                            while let Some(ch) = w.pop() {
                                if !ch.is_whitespace() { w.push(ch); break; }
                            }
                        }
                        self.stack.push(Variable::Text(v));
                        self.pop_fn(call.name.clone());
                        Expect::Something
                    }
                    "to_string" => {
                        self.push_fn(call.name.clone(), st + 1, lc);
                        let v = match self.stack.pop() {
                            Some(v) => v,
                            None => panic!("There is no value on the stack")
                        };
                        let v = match self.resolve(&v) {
                            &Variable::Text(ref t) => Variable::Text(t.clone()),
                            &Variable::F64(v) => {
                                Variable::Text(Arc::new(format!("{}", v)))
                            }
                            _ => unimplemented!(),
                        };
                        self.stack.push(v);
                        self.pop_fn(call.name.clone());
                        Expect::Something
                    }
                    "typeof" => {
                        self.push_fn(call.name.clone(), st + 1, lc);
                        let v = match self.stack.pop() {
                            Some(v) => v,
                            None => panic!("There is no value on the stack")
                        };
                        let v = match self.resolve(&v) {
                            &Variable::Text(_) => self.text_type.clone(),
                            &Variable::F64(_) => self.f64_type.clone(),
                            &Variable::Return => self.return_type.clone(),
                            &Variable::Bool(_) => self.bool_type.clone(),
                            &Variable::Object(_) => self.object_type.clone(),
                            &Variable::Array(_) => self.array_type.clone(),
                            &Variable::Ref(_) => self.ref_type.clone(),
                            &Variable::UnsafeRef(_) =>
                                self.unsafe_ref_type.clone(),
                            &Variable::RustObject(_) =>
                                self.rust_object_type.clone(),
                        };
                        self.stack.push(v);
                        self.pop_fn(call.name.clone());
                        Expect::Something
                    }
                    "debug" => {
                        self.push_fn(call.name.clone(), st, lc);
                        println!("Stack {:#?}", self.stack);
                        println!("Locals {:#?}", self.local_stack);
                        self.pop_fn(call.name.clone());
                        Expect::Nothing
                    }
                    "backtrace" => {
                        self.push_fn(call.name.clone(), st, lc);
                        println!("{:#?}", self.call_stack);
                        self.pop_fn(call.name.clone());
                        Expect::Nothing
                    }
                    "load" => {
                        use load;

                        self.push_fn(call.name.clone(), st + 1, lc);
                        let v = match self.stack.pop() {
                            Some(v) => v,
                            None => panic!("There is no value on the stack")
                        };
                        let v = match self.resolve(&v) {
                            &Variable::Text(ref text) => {
                                Variable::RustObject(Arc::new(Mutex::new(
                                    load(text, &[]).unwrap())))
                            }
                            _ => panic!("Expected text argument")
                        };
                        self.stack.push(v);
                        self.pop_fn(call.name.clone());
                        Expect::Something
                    }
                    "load_source_imports" => {
                        use load;

                        self.push_fn(call.name.clone(), st + 1, lc);
                        let modules = match self.stack.pop() {
                            Some(v) => v,
                            None => panic!("There is no value on the stack")
                        };
                        let source = match self.stack.pop() {
                            Some(v) => v,
                            None => panic!("There is no value on the stack")
                        };
                        let mut prelude: Vec<Arc<ast::Function>> = vec![];
                        match self.resolve(&modules) {
                            &Variable::Array(ref array) => {
                                for it in array {
                                    match self.resolve(it) {
                                        &Variable::RustObject(ref obj) => {
                                            match obj.lock().unwrap().downcast_ref::<Module>() {
                                                Some(m) => {
                                                    for f in m.functions.values() {
                                                        prelude.push(f.clone())
                                                    }
                                                }
                                                None => panic!("Expected `Module`")
                                            }
                                        }
                                        _ => panic!("Expected Rust object")
                                    }
                                }
                            }
                            _ => panic!("Expected array argument")
                        }
                        let v = match self.resolve(&source) {
                            &Variable::Text(ref text) => {
                                Variable::RustObject(Arc::new(Mutex::new(
                                load(text, &prelude).unwrap())))
                            }
                            _ => panic!("Expected text argument")
                        };
                        self.stack.push(v);
                        self.pop_fn(call.name.clone());
                        Expect::Something
                    }
                    "call" => {
                        self.push_fn(call.name.clone(), st, lc);
                        let args = match self.stack.pop() {
                            Some(v) => v,
                            None => panic!("There is no value on the stack")
                        };
                        let fn_name = match self.stack.pop() {
                            Some(v) => v,
                            None => panic!("There is no value on the stack")
                        };
                        let module = match self.stack.pop() {
                            Some(v) => v,
                            None => panic!("There is no value on the stack")
                        };
                        let fn_name = match self.resolve(&fn_name) {
                            &Variable::Text(ref text) => text.clone(),
                            _ => panic!("Expected text argument")
                        };
                        let args = match self.resolve(&args) {
                            &Variable::Array(ref arr) => arr.clone(),
                            _ => panic!("Expected array argument")
                        };
                        let obj = match self.resolve(&module) {
                            &Variable::RustObject(ref obj) => obj.clone(),
                            _ => panic!("Expected rust object")
                        };

                        match obj.lock().unwrap()
                            .downcast_ref::<Module>() {
                            Some(m) => {
                                match m.functions.get(&fn_name) {
                                    Some(ref f) => {
                                        if f.args.len() != args.len() {
                                            panic!("Expected `{}` arguments, found `{}`",
                                                f.args.len(), args.len())
                                        }
                                    }
                                    None => panic!("Could not find function `{}`", fn_name)
                                }
                                let call = ast::Call {
                                    name: fn_name.clone(),
                                    args: args.into_iter().map(|arg|
                                        ast::Expression::Variable(arg)).collect()
                                };
                                self.call(&call, &m);
                            }
                            None => panic!("Expected `Vec<ast::Function>`")
                        }

                        self.pop_fn(call.name.clone());
                        Expect::Nothing
                    }
                    _ => panic!("Unknown function `{}`", call.name)
                };
                (expect, Flow::Continue)
            }
            Some(ref f) => {
                if call.args.len() != f.args.len() {
                    panic!("Expected {} arguments but found {}", f.args.len(),
                        call.args.len());
                }
                // Arguments must be computed.
                if f.returns {
                    // Add return value before arguments on the stack.
                    // The stack value should remain, but the local should not.
                    self.stack.push(Variable::Return);
                }
                let st = self.stack.len();
                let lc = self.local_stack.len();
                for arg in &call.args {
                    match self.expression(arg, Side::Right, module) {
                        (x, Flow::Return) => { return (x, Flow::Return); }
                        (Expect::Something, Flow::Continue) => {}
                        _ => panic!("Expected something from argument")
                    };
                }
                self.push_fn(call.name.clone(), st, lc);
                if f.returns {
                    self.local_stack.push((self.ret.clone(), st - 1));
                }
                for (i, arg) in f.args.iter().enumerate() {
                    let j = st + i;
                    let j = match &self.stack[j] {
                        &Variable::Ref(ind) => ind,
                        _ => j
                    };
                    self.local_stack.push((arg.name.clone(), j));
                }
                match self.block(&f.block, module) {
                    (x, flow) => {
                        match flow {
                            Flow::Break(None) =>
                                panic!("Can not break from function"),
                            Flow::ContinueLoop(None) =>
                                panic!("Can not continue from function"),
                            Flow::Break(Some(ref label)) =>
                                panic!("There is no loop labeled `{}`", label),
                            Flow::ContinueLoop(Some(ref label)) =>
                                panic!("There is no loop labeled `{}`", label),
                            _ => {}
                        }
                        self.pop_fn(call.name.clone());
                        match (f.returns, x) {
                            (true, Expect::Nothing) => {
                                match self.stack.last() {
                                    Some(&Variable::Return) =>
                                        panic!("Function did not return a value"),
                                    None =>
                                        panic!("There is no value on the stack"),
                                    _ =>
                                        // This can happen when return is only
                                        // assigned to `return = x`.
                                        return (Expect::Something, Flow::Continue)
                                };
                            }
                            (false, Expect::Something) =>
                                panic!("Function `{}` should not return a value",
                                    f.name),
                            (true, Expect::Something)
                                if self.stack.len() == 0 =>
                                panic!("There is no value on the stack"),
                            (true, Expect::Something)
                                if match self.stack.last().unwrap() {
                                    &Variable::Return => true,
                                    _ => false
                                } =>
                                // TODO: Could return the last value on the stack.
                                //       Requires .pop_fn after.
                                panic!("Function did not return a value"),
                            (_, b) => {
                                return (b, Flow::Continue)
                            }
                        }
                    }
                }
            }
        }
    }

    fn object(&mut self, obj: &ast::Object, module: &Module) {
        let mut object: Object = HashMap::new();
        for &(ref key, ref expr) in &obj.key_values {
            self.expression(expr, Side::Right, module);
            match self.stack.pop() {
                None => panic!("There is no value on the stack"),
                Some(x) => {
                    match object.insert(key.clone(), x) {
                        None => {}
                        Some(_) => panic!("Duplicate key in object `{}`", key)
                    }
                }
            }
        }
        self.stack.push(Variable::Object(object));
    }

    fn array(&mut self, arr: &ast::Array, module: &Module) {
        let mut array: Array = Vec::new();
        for item in &arr.items {
            self.expression(item, Side::Right, module);
            match self.stack.pop() {
                None => panic!("There is no value on the stack"),
                Some(x) => array.push(x)
            }
        }
        self.stack.push(Variable::Array(array));
    }

    fn assign(&mut self, assign: &ast::Assign, module: &Module) -> Flow {
        self.assign_specific(assign.op, &assign.left, &assign.right, module)
    }

    fn assign_specific(
        &mut self,
        op: ast::AssignOp,
        left: &ast::Expression,
        right: &ast::Expression,
        module: &Module
    ) -> Flow {
        use ast::AssignOp::*;
        use ast::Expression;

        if op == Assign {
            match *left {
                Expression::Item(ref item) => {
                    match self.expression(right, Side::Right, module) {
                        (_, Flow::Return) => { return Flow::Return; }
                        (Expect::Something, Flow::Continue) => {}
                        _ => panic!("Expected something from the right side")
                    }
                    let v = match self.stack.pop() {
                        None => panic!("There is no value on the stack"),
                        // Use a shallow clone of a reference.
                        Some(Variable::Ref(ind)) => self.stack[ind].clone(),
                        Some(x) => x
                    };
                    if item.ids.len() != 0 {
                        match self.expression(left, Side::LeftInsert(true),
                                              module) {
                            (_, Flow::Return) => { return Flow::Return; }
                            (Expect::Something, Flow::Continue) => {}
                            _ => panic!("Expected something from the left side")
                        };
                        match self.stack.pop() {
                            Some(Variable::UnsafeRef(r)) => {
                                unsafe { *r = v }
                            }
                            None => panic!("There is no value on the stack"),
                            _ => panic!("Expected unsafe reference")
                        }
                    } else {
                        self.local_stack.push((item.name.clone(), self.stack.len()));
                        self.stack.push(v);
                    }
                    Flow::Continue
                }
                _ => panic!("Expected item")
            }
        } else {
            // Evaluate right side before left because the left leaves
            // an raw pointer on the stack which might point to wrong place
            // if there are side effects of the right side affecting it.
            match self.expression(right, Side::Right, module) {
                (_, Flow::Return) => { return Flow::Return; }
                (Expect::Something, Flow::Continue) => {}
                _ => panic!("Expected something from the right side")
            };
            match self.expression(left, Side::LeftInsert(false), module) {
                (_, Flow::Return) => { return Flow::Return; }
                (Expect::Something, Flow::Continue) => {}
                _ => panic!("Expected something from the left side")
            };
            match (self.stack.pop(), self.stack.pop()) {
                (Some(a), Some(b)) => {
                    let r = match a {
                        Variable::Ref(ind) => {
                            &mut self.stack[ind] as *mut Variable
                        }
                        Variable::UnsafeRef(r) => {
                            // If reference, use a shallow clone to type check,
                            // without affecting the original object.
                            unsafe {
                                if let Variable::Ref(ind) = *r {
                                    *r = self.stack[ind].clone()
                                }
                            }
                            r
                        }
                        x => panic!("Expected reference, found `{:?}`", x)
                    };

                    match self.resolve(&b) {
                        &Variable::F64(b) => {
                            unsafe {
                                match *r {
                                    Variable::F64(ref mut n) => {
                                        match op {
                                            Set => *n = b,
                                            Add => *n += b,
                                            Sub => *n -= b,
                                            Mul => *n *= b,
                                            Div => *n /= b,
                                            Rem => *n %= b,
                                            Pow => *n = n.powf(b),
                                            Assign => {}
                                        }
                                    }
                                    Variable::Return => {
                                        if let Set = op {
                                            *r = Variable::F64(b)
                                        } else {
                                            panic!("Return has no value")
                                        }
                                    }
                                    _ => panic!("Expected assigning to a number")
                                };
                            }
                        }
                        &Variable::Bool(b) => {
                            unsafe {
                                match *r {
                                    Variable::Bool(ref mut n) => {
                                        match op {
                                            Set => *n = b,
                                            _ => unimplemented!()
                                        }
                                    }
                                    Variable::Return => {
                                        if let Set = op {
                                            *r = Variable::Bool(b)
                                        } else {
                                            panic!("Return has no value")
                                        }
                                    }
                                    _ => panic!("Expected assigning to a bool")
                                };
                            }
                        }
                        &Variable::Text(ref b) => {
                            unsafe {
                                match *r {
                                    Variable::Text(ref mut n) => {
                                        match op {
                                            Set => *n = b.clone(),
                                            Add => Arc::make_mut(n).push_str(b),
                                            _ => unimplemented!()
                                        }
                                    }
                                    Variable::Return => {
                                        if let Set = op {
                                            *r = Variable::Text(b.clone())
                                        } else {
                                            panic!("Return has no value")
                                        }
                                    }
                                    _ => panic!("Expected assigning to text")
                                }
                            }
                        }
                        &Variable::Object(ref obj) => {
                            unsafe {
                                match *r {
                                    Variable::Object(ref mut n) => {
                                        if let Set = op {
                                            // Check address to avoid unsafe
                                            // reading and writing to same memory.
                                            let n_addr = n as *const _ as usize;
                                            let obj_addr = obj as *const _ as usize;
                                            if n_addr != obj_addr {
                                                *r = b.clone()
                                            }
                                            // *n = obj.clone()
                                        } else {
                                            unimplemented!()
                                        }
                                    }
                                    Variable::Return => {
                                        if let Set = op {
                                            *r = Variable::Object(obj.clone())
                                        } else {
                                            panic!("Return has no value")
                                        }
                                    }
                                    _ => panic!("Expected assigning to object")
                                }
                            }
                        }
                        &Variable::Array(ref arr) => {
                            unsafe {
                                match *r {
                                    Variable::Array(ref mut n) => {
                                        if let Set = op {
                                            // Check address to avoid unsafe
                                            // reading and writing to same memory.
                                            let n_addr = n as *const _ as usize;
                                            let arr_addr = arr as *const _ as usize;
                                            if n_addr != arr_addr {
                                                *r = b.clone()
                                            }
                                            // *n = arr.clone();
                                        } else {
                                            unimplemented!()
                                        }
                                    }
                                    Variable::Return => {
                                        if let Set = op {
                                            *r = Variable::Array(arr.clone())
                                        } else {
                                            panic!("Return has no value")
                                        }
                                    }
                                    _ => panic!("Expected assigning to array")
                                }
                            }
                        }
                        _ => unimplemented!()
                    };
                    Flow::Continue
                }
                _ => panic!("Expected two variables on the stack")
            }
        }
    }
    // `insert` is true for `:=` and false for `=`.
    // This works only on objects, but does not have to check since it is
    // ignored for arrays.
    fn item(&mut self, item: &ast::Item, side: Side, module: &Module) {
        use ast::Id;

        if item.ids.len() == 0 {
            let name: &str = &**item.name;
            let locals = self.local_stack.len() - self.call_stack.last().unwrap().2;
            for &(ref n, id) in self.local_stack.iter().rev().take(locals) {
                if &**n == name {
                    self.stack.push(Variable::Ref(id));
                    return;
                }
            }
            panic!("Could not find local variable `{}`", name);
        }

        // Pre-evalutate expressions for identity.
        let start_stack_len = self.stack.len();
        for id in &item.ids {
            if let &Id::Expression(ref expr) = id {
                self.expression(expr, Side::Right, module);
            }
        }
        let &mut Runtime {
            ref mut stack,
            ref mut local_stack,
            ref mut call_stack,
            ..
        } = self;
        let locals = local_stack.len() - call_stack.last().unwrap().2;
        let mut expr_j = 0;
        let name = &**item.name;
        let insert = match side {
            Side::Right => false,
            Side::LeftInsert(insert) => insert,
        };
        for &(ref n, id) in local_stack.iter().rev().take(locals) {
            if &**n != name { continue; }
            let v = {
                // Resolve reference of local variable.
                let id = if let &Variable::Ref(ref_id) = &stack[id] {
                        ref_id
                    } else {
                        id
                    };
                let item_len = item.ids.len();
                // Get the first variable (a.x).y
                let mut var: *mut Variable = item_lookup(
                    &mut stack[id],
                    stack,
                    &item.ids[0],
                    start_stack_len,
                    &mut expr_j,
                    insert,
                    item_len == 1
                );
                // Get the rest of the variables.
                for (i, prop) in item.ids[1..].iter().enumerate() {
                    var = item_lookup(
                        unsafe { &mut *var },
                        stack,
                        prop,
                        start_stack_len,
                        &mut expr_j,
                        insert,
                        // `i` skips first index.
                        i + 2 == item_len
                    );
                }

                match side {
                    Side::Right => unsafe {&*var}.clone(),
                    Side::LeftInsert(_) => Variable::UnsafeRef(var)
                }
            };
            stack.truncate(start_stack_len);
            stack.push(v);
            return;
        }
    }
    fn compare(&mut self, compare: &ast::Compare, module: &Module) -> Flow {
        match self.expression(&compare.left, Side::Right, module) {
            (_, Flow::Return) => { return Flow::Return; }
            (Expect::Something, Flow::Continue) => {}
            _ => panic!("Expected something from the left argument")
        };
        match self.expression(&compare.right, Side::Right, module) {
            (_, Flow::Return) => { return Flow::Return; }
            (Expect::Something, Flow::Continue) => {}
            _ => panic!("Expected something from the right argument")
        };
        match (self.stack.pop(), self.stack.pop()) {
            (Some(b), Some(a)) => {
                use ast::CompareOp::*;

                let v = match (self.resolve(&b), self.resolve(&a)) {
                    (&Variable::F64(b), &Variable::F64(a)) => {
                        Variable::Bool(match compare.op {
                            Less => a < b,
                            LessOrEqual => a <= b,
                            Greater => a > b,
                            GreaterOrEqual => a >= b,
                            Equal => a == b,
                            NotEqual => a != b
                        })
                    }
                    (&Variable::Text(ref b), &Variable::Text(ref a)) => {
                        Variable::Bool(match compare.op {
                            Less => a < b,
                            LessOrEqual => a <= b,
                            Greater => a > b,
                            GreaterOrEqual => a >= b,
                            Equal => a == b,
                            NotEqual => a != b
                        })
                    }
                    (&Variable::Bool(b), &Variable::Bool(a)) => {
                        Variable::Bool(match compare.op {
                            Less => panic!("`<` can not be used with bools"),
                            LessOrEqual => panic!("`<=` can not be used with bools"),
                            Greater => panic!("`>` can not be used with bools"),
                            GreaterOrEqual => panic!("`>=` can not be used with bools"),
                            Equal => a == b,
                            NotEqual => a != b
                        })
                    }
                    (b, a) => panic!("Invalid type `{:?}` `{:?}`", a, b)
                };
                self.stack.push(v)
            }
            _ => panic!("Expected two variables on the stack")
        }
        Flow::Continue
    }
    fn if_expr(&mut self, if_expr: &ast::If, module: &Module) -> (Expect, Flow) {
        match self.expression(&if_expr.cond, Side::Right, module) {
            (x, Flow::Return) => { return (x, Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => panic!("Expected bool from if condition")
        };
        match self.stack.pop() {
            None => panic!("There is no value on the stack"),
            Some(x) => match x {
                Variable::Bool(val) => {
                    if val {
                        return self.block(&if_expr.true_block, module);
                    }
                    for (cond, body) in if_expr.else_if_conds.iter()
                        .zip(if_expr.else_if_blocks.iter()) {
                        match self.expression(cond, Side::Right, module) {
                            (x, Flow::Return) => { return (x, Flow::Return); }
                            (Expect::Something, Flow::Continue) => {}
                            _ => panic!("Expected bool from else if condition")
                        };
                        match self.stack.pop() {
                            None => panic!("There is no value on the stack"),
                            Some(Variable::Bool(false)) => {}
                            Some(Variable::Bool(true)) => {
                                return self.block(body, module);
                            }
                            _ => panic!("Expected bool from else if condition")
                        }
                    }
                    if let Some(ref block) = if_expr.else_block {
                        self.block(block, module)
                    } else {
                        (Expect::Nothing, Flow::Continue)
                    }
                }
                _ => panic!("Expected bool")
            }
        }
    }
    fn for_expr(&mut self, for_expr: &ast::For, module: &Module) -> Flow {
        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();
        self.expression(&for_expr.init, Side::Right, module);
        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            self.expression(&for_expr.cond, Side::Right, module);
            match self.stack.pop() {
                None => panic!("There is no value on the stack"),
                Some(x) => match x {
                    Variable::Bool(val) => {
                        if val {
                            match self.block(&for_expr.block, module) {
                                (_, Flow::Return) => { return Flow::Return; }
                                (_, Flow::Continue) => {}
                                (_, Flow::Break(x)) => {
                                    match x {
                                        Some(label) => {
                                            let same =
                                            if let Some(ref for_label) = for_expr.label {
                                                &label == for_label
                                            } else { false };
                                            if !same {
                                                flow = Flow::Break(Some(label))
                                            }
                                        }
                                        None => {}
                                    }
                                    break;
                                }
                                (_, Flow::ContinueLoop(x)) => {
                                    match x {
                                        Some(label) => {
                                            let same =
                                            if let Some(ref for_label) = for_expr.label {
                                                &label == for_label
                                            } else { false };
                                            if !same {
                                                flow = Flow::ContinueLoop(Some(label));
                                                break;
                                            }
                                        }
                                        None => {}
                                    }
                                    self.expression(&for_expr.step, Side::Right,
                                                    module);
                                    continue;
                                }
                            }
                            self.expression(&for_expr.step, Side::Right,
                                            module);
                        } else {
                            break;
                        }
                    }
                    _ => panic!("Expected bool")
                }
            };
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        flow
    }
    fn text(&mut self, text: &ast::Text) {
        self.stack.push(Variable::Text(text.text.clone()));
    }
    fn number(&mut self, num: &ast::Number) {
        self.stack.push(Variable::F64(num.num));
    }
    fn bool(&mut self, val: &ast::Bool) {
        self.stack.push(Variable::Bool(val.val));
    }
    fn unop(
        &mut self,
        unop: &ast::UnOpExpression,
        side: Side,
        module: &Module
    ) -> Flow {
        match self.expression(&unop.expr, side, module) {
            (_, Flow::Return) => { return Flow::Return; }
            (Expect::Something, Flow::Continue) => {}
            _ => panic!("Expected something from unary argument")
        };
        let val = self.stack.pop().expect("Expected unary argument");
        let v = match self.resolve(&val) {
            &Variable::Bool(b) => {
                Variable::Bool(match unop.op {
                    ast::UnOp::Neg => !b,
                    // _ => panic!("Unknown boolean unary operator `{:?}`", unop.op)
                })
            }
            _ => panic!("Invalid type, expected bool")
        };
        self.stack.push(v);
        Flow::Continue
    }
    fn binop(
        &mut self,
        binop: &ast::BinOpExpression,
        side: Side,
        module: &Module
    ) -> Flow {
        use ast::BinOp::*;

        match self.expression(&binop.left, side, module) {
            (_, Flow::Return) => { return Flow::Return; }
            (Expect::Something, Flow::Continue) => {}
            _ => panic!("Expected something from left argument")
        };
        match self.expression(&binop.right, side, module) {
            (_, Flow::Return) => { return Flow::Return; }
            (Expect::Something, Flow::Continue) => {}
            _ => panic!("Expected something from right argument")
        };
        let right = self.stack.pop().expect("Expected right argument");
        let left = self.stack.pop().expect("Expected left argument");
        let v = match (self.resolve(&left), self.resolve(&right)) {
            (&Variable::F64(a), &Variable::F64(b)) => {
                Variable::F64(match binop.op {
                    Add => a + b,
                    Sub => a - b,
                    Mul => a * b,
                    Div => a / b,
                    Rem => a % b,
                    Pow => a.powf(b)
                })
            }
            (&Variable::Bool(a), &Variable::Bool(b)) => {
                Variable::Bool(match binop.op {
                    Add => a || b,
                    // Boolean subtraction with lazy precedence.
                    Sub => a && !b,
                    Mul => a && b,
                    Pow => a ^ b,
                    _ => panic!("Unknown boolean operator `{:?}`", binop.op)
                })
            }
            (&Variable::Text(ref a), &Variable::Text(ref b)) => {
                match binop.op {
                    Add => {
                        let mut res = String::with_capacity(a.len() + b.len());
                        res.push_str(a);
                        res.push_str(b);
                        Variable::Text(Arc::new(res))
                    }
                    _ => panic!("This operation can not be used with strings")
                }
            }
            (&Variable::Text(_), _) =>
                panic!("The right argument must be a string. Try the `to_string` function"),
            _ => panic!("Invalid type, expected numbers, bools or strings")
        };
        self.stack.push(v);

        Flow::Continue
    }
}
