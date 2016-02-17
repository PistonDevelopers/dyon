use std::sync::Arc;
use std::collections::HashMap;
use rand;

use ast;
use intrinsics;

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

    #[inline(always)]
    pub fn resolve<'a>(&'a self, var: &'a Variable) -> &'a Variable {
        resolve(&self.stack, var)
    }

    pub fn unary_f64<F: FnOnce(f64) -> f64>(&mut self, f: F) -> Expect {
        let x = self.stack.pop().expect("There is no value on the stack");
        match self.resolve(&x) {
            &Variable::F64(a) => {
                self.stack.push(Variable::F64(f(a)));
            }
            _ => panic!("Expected number")
        }
        Expect::Something
    }

    #[inline(always)]
    pub fn push_fn(&mut self, name: Arc<String>, st: usize, lc: usize) {
        self.call_stack.push((name, st, lc));
    }
    pub fn pop_fn(&mut self, name: Arc<String>) {
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

    pub fn expression(
        &mut self,
        expr: &ast::Expression,
        side: Side,
        module: &Module
    ) -> Result<(Expect, Flow), String> {
        use ast::Expression::*;

        match *expr {
            Object(ref obj) => {
                let flow = try!(self.object(obj, module));
                Ok((Expect::Something, flow))
            }
            Array(ref arr) => {
                let flow = try!(self.array(arr, module));
                Ok((Expect::Something, flow))
            }
            ArrayFill(ref array_fill) => {
                let flow = try!(self.array_fill(array_fill, module));
                Ok((Expect::Something, flow))
            }
            Block(ref block) => self.block(block, module),
            Return(ref ret) => {
                use ast::{AssignOp, Expression, Item};

                // Assign return value and then break the flow.
                let item = Expression::Item(Item {
                        name: self.ret.clone(),
                        ids: vec![]
                    });
                let flow = try!(self.assign_specific(AssignOp::Set,
                    &item, ret, module));
                Ok((Expect::Something, flow))
            }
            ReturnVoid => {
                Ok((Expect::Nothing, Flow::Return))
            }
            Break(ref b) => Ok((Expect::Nothing, Flow::Break(b.label.clone()))),
            Continue(ref b) => Ok((Expect::Nothing,
                                   Flow::ContinueLoop(b.label.clone()))),
            Call(ref call) => self.call(call, module),
            Item(ref item) => {
                let flow = try!(self.item(item, side, module));
                Ok((Expect::Something, flow))
            }
            UnOp(ref unop) => Ok((Expect::Something,
                                  try!(self.unop(unop, side, module)))),
            BinOp(ref binop) => Ok((Expect::Something,
                                    try!(self.binop(binop, side, module)))),
            Assign(ref assign) => Ok((Expect::Nothing,
                                      try!(self.assign(assign, module)))),
            Number(ref num) => {
                self.number(num);
                Ok((Expect::Something, Flow::Continue))
            }
            Text(ref text) => {
                self.text(text);
                Ok((Expect::Something, Flow::Continue))
            }
            Bool(ref b) => {
                self.bool(b);
                Ok((Expect::Something, Flow::Continue))
            }
            For(ref for_expr) => Ok((Expect::Nothing,
                                     try!(self.for_expr(for_expr, module)))),
            If(ref if_expr) => self.if_expr(if_expr, module),
            Compare(ref compare) => Ok((Expect::Something,
                                        try!(self.compare(compare, module)))),
            Variable(ref var) => {
                self.stack.push(var.clone());
                Ok((Expect::Something, Flow::Continue))
            }
        }
    }

    pub fn run(&mut self, module: &Module) -> Result<(), String> {
        let call = ast::Call {
            name: Arc::new("main".into()),
            args: vec![]
        };
        match module.functions.get(&call.name) {
            Some(f) => {
                if f.args.len() != 0 {
                    panic!("`main` should not have arguments");
                }
                try!(self.call(&call, &module));
                Ok(())
            }
            None => panic!("Could not find function `main`")
        }
    }

    fn block(
        &mut self,
        block: &ast::Block,
        module: &Module
    ) -> Result<(Expect, Flow), String> {
        let mut expect = Expect::Nothing;
        let lc = self.local_stack.len();
        for e in &block.expressions {
            expect = match try!(self.expression(e, Side::Right, module)) {
                (x, Flow::Continue) => x,
                x => { return Ok(x); }
            }
        }
        self.local_stack.truncate(lc);
        Ok((expect, Flow::Continue))
    }

    pub fn call(
        &mut self,
        call: &ast::Call,
        module: &Module
    ) -> Result<(Expect, Flow), String> {
        match module.functions.get(&call.name) {
            None => {
                intrinsics::call_standard(self, call, module)
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
                    match try!(self.expression(arg, Side::Right, module)) {
                        (x, Flow::Return) => { return Ok((x, Flow::Return)); }
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
                match try!(self.block(&f.block, module)) {
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
                                        return Ok((Expect::Something,
                                                   Flow::Continue))
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
                                return Ok((b, Flow::Continue))
                            }
                        }
                    }
                }
            }
        }
    }

    fn object(
        &mut self,
        obj: &ast::Object,
        module: &Module
    ) -> Result<Flow, String> {
        let mut object: Object = HashMap::new();
        for &(ref key, ref expr) in &obj.key_values {
            match try!(self.expression(expr, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => panic!("Expected something")
            };
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
        Ok(Flow::Continue)
    }

    fn array(
        &mut self,
        arr: &ast::Array,
        module: &Module
    ) -> Result<Flow, String> {
        let mut array: Array = Vec::new();
        for item in &arr.items {
            match try!(self.expression(item, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => panic!("Expected something")
            };
            match self.stack.pop() {
                None => panic!("There is no value on the stack"),
                Some(x) => array.push(x)
            }
        }
        self.stack.push(Variable::Array(array));
        Ok(Flow::Continue)
    }

    fn array_fill(
        &mut self,
        array_fill: &ast::ArrayFill,
        module: &Module
    ) -> Result<Flow, String> {
        match try!(self.expression(&array_fill.fill, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => panic!("Expected something")
        };
        match try!(self.expression(&array_fill.n, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => panic!("Expected something")
        };
        match (self.stack.pop(), self.stack.pop()) {
            (None, _) | (_, None) => panic!("There is no value on the stack"),
            (Some(Variable::F64(n)), Some(x)) => {
                self.stack.push(Variable::Array(vec![x; n as usize]));
            }
            _ => panic!("Expected number for length in `[value; length]`")
        }
        Ok(Flow::Continue)
    }

    #[inline(always)]
    fn assign(
        &mut self,
        assign: &ast::Assign,
        module: &Module
    ) -> Result<Flow, String> {
        self.assign_specific(assign.op, &assign.left, &assign.right, module)
    }

    fn assign_specific(
        &mut self,
        op: ast::AssignOp,
        left: &ast::Expression,
        right: &ast::Expression,
        module: &Module
    ) -> Result<Flow, String> {
        use ast::AssignOp::*;
        use ast::Expression;

        if op == Assign {
            match *left {
                Expression::Item(ref item) => {
                    match try!(self.expression(right, Side::Right, module)) {
                        (_, Flow::Return) => { return Ok(Flow::Return); }
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
                        match try!(self.expression(left, Side::LeftInsert(true),
                                                   module)) {
                            (_, Flow::Return) => { return Ok(Flow::Return); }
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
                    Ok(Flow::Continue)
                }
                _ => panic!("Expected item")
            }
        } else {
            // Evaluate right side before left because the left leaves
            // an raw pointer on the stack which might point to wrong place
            // if there are side effects of the right side affecting it.
            match try!(self.expression(right, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => panic!("Expected something from the right side")
            };
            match try!(self.expression(left, Side::LeftInsert(false), module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
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
                    Ok(Flow::Continue)
                }
                _ => panic!("Expected two variables on the stack")
            }
        }
    }
    // `insert` is true for `:=` and false for `=`.
    // This works only on objects, but does not have to check since it is
    // ignored for arrays.
    fn item(
        &mut self,
        item: &ast::Item,
        side: Side,
        module: &Module
    ) -> Result<Flow, String> {
        use ast::Id;

        if item.ids.len() == 0 {
            let name: &str = &**item.name;
            let locals = self.local_stack.len() - self.call_stack.last().unwrap().2;
            for &(ref n, id) in self.local_stack.iter().rev().take(locals) {
                if &**n == name {
                    self.stack.push(Variable::Ref(id));
                    return Ok(Flow::Continue);
                }
            }
            panic!("Could not find local variable `{}`", name);
        }

        // Pre-evalutate expressions for identity.
        let start_stack_len = self.stack.len();
        for id in &item.ids {
            if let &Id::Expression(ref expr) = id {
                match try!(self.expression(expr, Side::Right, module)) {
                    (_, Flow::Return) => { return Ok(Flow::Return); }
                    (Expect::Something, Flow::Continue) => {}
                    _ => panic!("Expected something for index")
                };
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
            break;
        }
        return Ok(Flow::Continue);
    }
    fn compare(
        &mut self,
        compare: &ast::Compare,
        module: &Module
    ) -> Result<Flow, String> {
        match try!(self.expression(&compare.left, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => panic!("Expected something from the left argument")
        };
        match try!(self.expression(&compare.right, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
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
        Ok(Flow::Continue)
    }
    fn if_expr(
        &mut self,
        if_expr: &ast::If,
        module: &Module
    ) -> Result<(Expect, Flow), String> {
        match try!(self.expression(&if_expr.cond, Side::Right, module)) {
            (x, Flow::Return) => { return Ok((x, Flow::Return)); }
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
                        match try!(self.expression(cond, Side::Right, module)) {
                            (x, Flow::Return) => {
                                return Ok((x, Flow::Return));
                            }
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
                        Ok((Expect::Nothing, Flow::Continue))
                    }
                }
                _ => panic!("Expected bool")
            }
        }
    }
    fn for_expr(
        &mut self,
        for_expr: &ast::For,
        module: &Module
    ) -> Result<Flow, String> {
        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();
        match try!(self.expression(&for_expr.init, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Nothing, Flow::Continue) => {}
            _ => panic!("Expected nothing from for init")
        };
        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match try!(self.expression(&for_expr.cond, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => panic!("Expected bool from for condition")
            };
            match self.stack.pop() {
                None => panic!("There is no value on the stack"),
                Some(x) => match x {
                    Variable::Bool(val) => {
                        if val {
                            match try!(self.block(&for_expr.block, module)) {
                                (_, Flow::Return) => {
                                    return Ok(Flow::Return);
                                }
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
                                    match try!(self.expression(
                                        &for_expr.step, Side::Right, module)) {
                                            (_, Flow::Return) => {
                                                return Ok(Flow::Return);
                                            }
                                            (Expect::Nothing, Flow::Continue) => {}
                                            _ => panic!("Expected nothing from for step")
                                    };
                                    continue;
                                }
                            }
                            match try!(self.expression(
                                &for_expr.step, Side::Right, module)) {
                                    (_, Flow::Return) => {
                                        return Ok(Flow::Return);
                                    }
                                    (Expect::Nothing, Flow::Continue) => {}
                                    _ => panic!("Expected nothing from for step")
                            };
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
        Ok(flow)
    }
    #[inline(always)]
    fn text(&mut self, text: &ast::Text) {
        self.stack.push(Variable::Text(text.text.clone()));
    }
    #[inline(always)]
    fn number(&mut self, num: &ast::Number) {
        self.stack.push(Variable::F64(num.num));
    }
    #[inline(always)]
    fn bool(&mut self, val: &ast::Bool) {
        self.stack.push(Variable::Bool(val.val));
    }
    fn unop(
        &mut self,
        unop: &ast::UnOpExpression,
        side: Side,
        module: &Module
    ) -> Result<Flow, String> {
        match try!(self.expression(&unop.expr, side, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
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
        Ok(Flow::Continue)
    }
    fn binop(
        &mut self,
        binop: &ast::BinOpExpression,
        side: Side,
        module: &Module
    ) -> Result<Flow, String> {
        use ast::BinOp::*;

        match try!(self.expression(&binop.left, side, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(binop.source_range,
                "Expected something from left argument"))
        };
        match try!(self.expression(&binop.right, side, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(binop.source_range,
                "Expected something from right argument"))
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
                    _ => return Err(module.error(binop.source_range,
                        &format!("Unknown boolean operator `{:?}`",
                            binop.op.symbol_bool())))
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
                    _ => return Err(module.error(binop.source_range,
                        "This operation can not be used with strings"))
                }
            }
            (&Variable::Text(_), _) =>
                return Err(module.error(binop.source_range,
                "The right argument must be a string. \
                Try the `to_string` function")),
            _ => return Err(module.error(binop.source_range, &format!(
                "Invalid type for binary operator `{:?}`, \
                expected numbers, bools or strings",
                binop.op.symbol())))
        };
        self.stack.push(v);

        Ok(Flow::Continue)
    }
}
