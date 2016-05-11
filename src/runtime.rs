use std::sync::Arc;
use std::collections::HashMap;
use rand;
use range::Range;

use ast;
use intrinsics;
use embed;

use Variable;
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Call {
    // was .0
    pub fn_name: Arc<String>,
    pub file: Option<Arc<String>>,
    // was .1
    pub stack_len: usize,
    // was .2
    pub local_len: usize,
}

pub struct Runtime {
    pub stack: Vec<Variable>,
    /// name, file, stack_len, local_len.
    pub call_stack: Vec<Call>,
    pub local_stack: Vec<(Arc<String>, usize)>,
    pub ret: Arc<String>,
    pub rng: rand::ThreadRng,
    pub text_type: Variable,
    pub f64_type: Variable,
    pub vec4_type: Variable,
    pub return_type: Variable,
    pub bool_type: Variable,
    pub object_type: Variable,
    pub array_type: Variable,
    pub ref_type: Variable,
    pub unsafe_ref_type: Variable,
    pub rust_object_type: Variable,
    pub option_type: Variable,
    pub result_type: Variable,
}

fn resolve<'a>(stack: &'a Vec<Variable>, var: &'a Variable) -> &'a Variable {
    match *var {
        Variable::Ref(ind) => &stack[ind],
        _ => var
    }
}

// Looks up an item from a variable property.
fn item_lookup(
    module: &Module,
    var: *mut Variable,
    stack: &mut [Variable],
    call_stack: &[Call],
    prop: &ast::Id,
    start_stack_len: usize,
    expr_j: &mut usize,
    insert: bool, // Whether to insert key in object.
    last: bool,   // Whether it is the last property.
) -> Result<*mut Variable, String> {
    use ast::Id;
    use std::collections::hash_map::Entry;

    unsafe {
        match *var {
            Variable::Object(ref mut obj) => {
                let id = match prop {
                    &Id::String(_, ref id) => id.clone(),
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
                            _ => return Err(module.error(prop.source_range(),
                                &format!("{}\nExpected string",
                                    stack_trace(call_stack))))
                        }
                    }
                    &Id::F64(range, _) => return Err(module.error(range,
                        &format!("{}\nExpected string",
                            stack_trace(call_stack))))
                };
                let v = match Arc::make_mut(obj).entry(id.clone()) {
                    Entry::Vacant(vac) => {
                        if insert && last {
                            // Insert a key to overwrite with new value.
                            vac.insert(Variable::Return)
                        } else {
                            return Err(module.error(prop.source_range(),
                                &format!("{}\nObject has no key `{}`",
                                    stack_trace(call_stack), id)));
                        }
                    }
                    Entry::Occupied(v) => v.into_mut()
                };
                // Resolve reference.
                if let &mut Variable::Ref(id) = v {
                    // Do not resolve if last, because references should be
                    // copy-on-write.
                    if last {
                        Ok(v)
                    } else {
                        Ok(&mut stack[id])
                    }
                } else {
                    Ok(v)
                }
            }
            Variable::Array(ref mut arr) => {
                let id = match prop {
                    &Id::F64(_, id) => id,
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
                            _ => return Err(module.error(prop.source_range(),
                                            &format!("{}\nExpected number",
                                                stack_trace(call_stack))))
                        }
                    }
                    &Id::String(range, _) => return Err(module.error(range,
                        &format!("{}\nExpected number",
                            stack_trace(call_stack))))
                };
                let v = match Arc::make_mut(arr).get_mut(id as usize) {
                    None => return Err(module.error(prop.source_range(),
                                       &format!("{}\nOut of bounds `{}`",
                                                stack_trace(call_stack), id))),
                    Some(x) => x
                };
                // Resolve reference.
                if let &mut Variable::Ref(id) = v {
                    // Do not resolve if last, because references should be
                    // copy-on-write.
                    if last {
                        Ok(v)
                    } else {
                        Ok(&mut stack[id])
                    }
                } else {
                    Ok(v)
                }
            }
            _ => return Err(module.error(prop.source_range(),
                            &format!("{}\nLook up requires object or array",
                            stack_trace(call_stack))))
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
            vec4_type: Variable::Text(Arc::new("vec4".into())),
            return_type: Variable::Text(Arc::new("return".into())),
            bool_type: Variable::Text(Arc::new("boolean".into())),
            object_type: Variable::Text(Arc::new("object".into())),
            array_type: Variable::Text(Arc::new("array".into())),
            ref_type: Variable::Text(Arc::new("ref".into())),
            unsafe_ref_type: Variable::Text(Arc::new("unsafe_ref".into())),
            rust_object_type: Variable::Text(Arc::new("rust_object".into())),
            option_type: Variable::Text(Arc::new("option".into())),
            result_type: Variable::Text(Arc::new("result".into())),
        }
    }

    pub fn pop<T: embed::PopVariable>(&mut self) -> Result<T, String> {
        let v = self.stack.pop().unwrap_or_else(|| {
            panic!("There is no value on the stack")
        });
        T::pop_var(self, self.resolve(&v))
    }

    pub fn pop_vec4<T: embed::ConvertVec4>(&mut self) -> Result<T, String> {
        let v = self.stack.pop().unwrap_or_else(|| {
            panic!("There is no value on the stack")
        });
        match self.resolve(&v) {
            &Variable::Vec4(val) => Ok(T::from(val)),
            x => Err(self.expected(x, "vec4"))
        }
    }

    pub fn var<T: embed::PopVariable>(&self, var: &Variable) -> Result<T, String> {
        T::pop_var(self, self.resolve(&var))
    }

    pub fn var_vec4<T: embed::ConvertVec4>(&self, var: &Variable) -> Result<T, String> {
        match self.resolve(&var) {
            &Variable::Vec4(val) => Ok(T::from(val)),
            x => Err(self.expected(x, "vec4"))
        }
    }

    pub fn push<T: embed::PushVariable>(&mut self, val: T) {
        self.stack.push(val.push_var())
    }

    pub fn push_vec4<T: embed::ConvertVec4>(&mut self, val: T) {
        self.stack.push(Variable::Vec4(val.to()))
    }

    pub fn expected(&self, var: &Variable, ty: &str) -> String {
        let found_ty = self.typeof_var(var);
        format!("{}\nExpected `{}`, found `{}`", self.stack_trace(), ty, found_ty)
    }

    #[inline(always)]
    pub fn resolve<'a>(&'a self, var: &'a Variable) -> &'a Variable {
        resolve(&self.stack, var)
    }

    pub fn unary_f64<F: FnOnce(f64) -> f64>(
        &mut self,
        call: &ast::Call,
        module: &Module,
        f: F
    ) -> Result<Expect, String> {
        let x = self.stack.pop().expect("There is no value on the stack");
        match self.resolve(&x) {
            &Variable::F64(a) => {
                self.stack.push(Variable::F64(f(a)));
            }
            _ => return Err(module.error(call.args[0].source_range(),
                    &format!("{}\nExpected number", self.stack_trace())))
        }
        Ok(Expect::Something)
    }

    #[inline(always)]
    pub fn push_fn(
        &mut self,
        name: Arc<String>,
        file: Option<Arc<String>>,
        st: usize,
        lc: usize
    ) {
        self.call_stack.push(Call {
            fn_name: name,
            file: file,
            stack_len: st,
            local_len: lc
        });
    }
    pub fn pop_fn(&mut self, name: Arc<String>) {
        match self.call_stack.pop() {
            None => panic!("Did not call `{}`", name),
            Some(Call { fn_name, stack_len: st, local_len: lc, .. }) => {
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
                        try: false,
                        ids: vec![],
                        try_ids: vec![],
                        source_range: ret.source_range(),
                    });
                let _flow = try!(self.assign_specific(AssignOp::Set,
                    &item, ret, module));
                Ok((Expect::Something, Flow::Return))
            }
            ReturnVoid(_) => {
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
            Vec4(ref vec4) => {
                Ok((Expect::Something, try!(self.vec4(vec4, side, module))))
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
            ForN(ref for_n_expr) => Ok((Expect::Nothing,
                                     try!(self.for_n_expr(for_n_expr, module)))),
            Sum(ref for_n_expr) => Ok((Expect::Something,
                                     try!(self.sum_n_expr(for_n_expr, module)))),
            Min(ref for_n_expr) => Ok((Expect::Something,
                                     try!(self.min_n_expr(for_n_expr, module)))),
            Max(ref for_n_expr) => Ok((Expect::Something,
                                     try!(self.max_n_expr(for_n_expr, module)))),
            Sift(ref for_n_expr) => Ok((Expect::Something,
                                     try!(self.sift_n_expr(for_n_expr, module)))),
            Any(ref for_n_expr) => Ok((Expect::Something,
                                     try!(self.any_n_expr(for_n_expr, module)))),
            All(ref for_n_expr) => Ok((Expect::Something,
                                     try!(self.all_n_expr(for_n_expr, module)))),
            If(ref if_expr) => self.if_expr(if_expr, module),
            Compare(ref compare) => Ok((Expect::Something,
                                        try!(self.compare(compare, module)))),
            Variable(_, ref var) => {
                self.stack.push(var.clone());
                Ok((Expect::Something, Flow::Continue))
            }
            Try(ref expr) => {
                self.try(expr, side, module)
            }
        }
    }

    pub fn try(
        &mut self,
        expr: &ast::Expression,
        side: Side,
        module: &Module
    ) -> Result<(Expect, Flow), String> {
        use Error;

        match self.expression(expr, side, module) {
            Ok((x, Flow::Return)) => { return Ok((x, Flow::Return)); }
            Ok((Expect::Something, Flow::Continue)) => {}
            _ => return Err(module.error(expr.source_range(),
                            &format!("{}\nExpected something",
                                self.stack_trace())))
        };
        let v = self.stack.pop()
            .expect("There is no value on the stack");
        let v = match self.resolve(&v) {
            &Variable::Result(ref res) => res.clone(),
            &Variable::Option(ref opt) => {
                match opt {
                    &Some(ref some) => Ok(some.clone()),
                    &None => Err(Box::new(Error {
                        message: Variable::Text(Arc::new(
                            "Expected `some(_)`, found `none()`"
                            .into())),
                        trace: vec![]
                    }))
                }
            }
            _ => {
                return Err(module.error(expr.source_range(),
                    &format!("{}\nExpected `ok(_)` or `err(_)`",
                        self.stack_trace())));
            }
        };
        let locals = self.local_stack.len() - self.call_stack.last().unwrap().local_len;
        match v {
            Ok(ref ok) => {
                self.stack.push((**ok).clone());
                Ok((Expect::Something, Flow::Continue))
            }
            Err(ref err) => {
                let ind = self.stack.len() - locals;
                if locals == 0 {
                    return Err(module.error(expr.source_range(),
                        &format!("{}\nRequires `->` on function `{}`",
                        self.stack_trace(),
                        &self.call_stack.last().unwrap().fn_name)));
                }
                if let Variable::Return = self.stack[ind] {}
                else {
                    return Err(module.error(expr.source_range(),
                        &format!("{}\nRequires `->` on function `{}`",
                        self.stack_trace(),
                        &self.call_stack.last().unwrap().fn_name)));
                }
                let mut err = err.clone();
                let call = self.call_stack.last().unwrap();
                let file = match call.file {
                    None => "".into(),
                    Some(ref f) => format!(" ({})", f)
                };
                err.trace.push(module.error(expr.source_range(),
                    &format!("In function `{}`{}",
                    &call.fn_name, file)));
                self.stack[ind] = Variable::Result(Err(err));
                Ok((Expect::Something, Flow::Return))
            }
        }
    }

    pub fn run(&mut self, module: &Module) -> Result<(), String> {
        let call = ast::Call {
            name: Arc::new("main".into()),
            args: vec![],
            source_range: Range::empty(0),
        };
        match module.functions.get(&call.name) {
            Some(f) => {
                if f.args.len() != 0 {
                    return Err(module.error(f.args[0].source_range,
                               "`main` should not have arguments"))
                }
                try!(self.call(&call, &module));
                Ok(())
            }
            None => return Err(module.error(call.source_range,
                               "Could not find function `main`"))
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
                match module.ext_prelude.get(&call.name) {
                    None => {
                        intrinsics::call_standard(self, call, module)
                    }
                    Some(&(ref f, ref pr)) => {
                        for arg in &call.args {
                            match try!(self.expression(arg, Side::Right, module)) {
                                (x, Flow::Return) => { return Ok((x, Flow::Return)); }
                                (Expect::Something, Flow::Continue) => {}
                                _ => return Err(module.error(arg.source_range(),
                                                &format!("{}\nExpected something. \
                                                Expression did not return a value.",
                                                self.stack_trace())))
                            };
                        }
                        try!(f(self).map_err(|err|
                            module.error(call.source_range, &err)));
                        if pr.returns() {
                            return Ok((Expect::Something, Flow::Continue));
                        } else {
                            return Ok((Expect::Nothing, Flow::Continue));
                        }
                    }
                }
            }
            Some(ref f) => {
                if call.args.len() != f.args.len() {
                    return Err(module.error(call.source_range,
                        &format!("{}\nExpected {} arguments but found {}",
                        self.stack_trace(),
                        f.args.len(),
                        call.args.len())));
                }
                // Arguments must be computed.
                if f.returns() {
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
                        _ => return Err(module.error(arg.source_range(),
                                        &format!("{}\nExpected something. \
                                        Check that expression returns a value.",
                                        self.stack_trace())))
                    };
                }
                self.push_fn(call.name.clone(), Some(f.file.clone()), st, lc);
                if f.returns() {
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
                                return Err(module.error(call.source_range,
                                           &format!("{}\nCan not break from function",
                                                self.stack_trace()))),
                            Flow::ContinueLoop(None) =>
                                return Err(module.error(call.source_range,
                                           &format!("{}\nCan not continue from function",
                                                self.stack_trace()))),
                            Flow::Break(Some(ref label)) =>
                                return Err(module.error(call.source_range,
                                    &format!("{}\nThere is no loop labeled `{}`",
                                             self.stack_trace(), label))),
                            Flow::ContinueLoop(Some(ref label)) =>
                                return Err(module.error(call.source_range,
                                    &format!("{}\nThere is no loop labeled `{}`",
                                            self.stack_trace(), label))),
                            _ => {}
                        }
                        self.pop_fn(call.name.clone());
                        match (f.returns(), x) {
                            (true, Expect::Nothing) => {
                                match self.stack.last() {
                                    Some(&Variable::Return) =>
                                        return Err(module.error(
                                        call.source_range, &format!(
                                        "{}\nFunction `{}` did not return a value",
                                        self.stack_trace(),
                                        f.name))),
                                    None =>
                                        panic!("There is no value on the stack"),
                                    _ => {
                                        // This happens when return is only
                                        // assigned to `return = x`.
                                        return Ok((Expect::Something,
                                                   Flow::Continue))
                                    }
                                };
                            }
                            (false, Expect::Something) =>
                                return Err(module.error(call.source_range,
                                    &format!(
                                        "{}\nFunction `{}` should not return a value",
                                        self.stack_trace(),
                                        f.name))),
                            (true, Expect::Something)
                                if self.stack.len() == 0 =>
                                panic!("There is no value on the stack"),
                            (true, Expect::Something)
                                if match self.stack.last().unwrap() {
                                    &Variable::Return => true,
                                    _ => false
                                } =>
                                // TODO: Could return the last value on the stack.
                                //       Requires .pop_fn delayed after.
                                return Err(module.error(call.source_range,
                                    &format!(
                                    "{}\nFunction `{}` did not return a value. \
                                    Did you forgot a `return`?",
                                        self.stack_trace(),
                                        f.name))),
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
        let mut object: HashMap<_, _> = HashMap::new();
        for &(ref key, ref expr) in &obj.key_values {
            match try!(self.expression(expr, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(expr.source_range(),
                                &format!("{}\nExpected something",
                                    self.stack_trace())))
            };
            match self.stack.pop() {
                None => panic!("There is no value on the stack"),
                Some(x) => {
                    match object.insert(key.clone(), x) {
                        None => {}
                        Some(_) => return Err(module.error(expr.source_range(),
                            &format!("{}\nDuplicate key in object `{}`",
                                self.stack_trace(), key)))
                    }
                }
            }
        }
        self.stack.push(Variable::Object(Arc::new(object)));
        Ok(Flow::Continue)
    }

    fn array(
        &mut self,
        arr: &ast::Array,
        module: &Module
    ) -> Result<Flow, String> {
        let mut array: Vec<Variable> = Vec::new();
        for item in &arr.items {
            match try!(self.expression(item, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(item.source_range(),
                    &format!("{}\nExpected something",
                        self.stack_trace())))
            };
            match self.stack.pop() {
                None => panic!("There is no value on the stack"),
                Some(x) => array.push(x)
            }
        }
        self.stack.push(Variable::Array(Arc::new(array)));
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
            _ => return Err(module.error(array_fill.fill.source_range(),
                            &format!("{}\nExpected something",
                                self.stack_trace())))
        };
        match try!(self.expression(&array_fill.n, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(array_fill.n.source_range(),
                            &format!("{}\nExpected something",
                                self.stack_trace())))
        };
        let n: Variable = self.stack.pop().expect("Expected n");
        let fill: Variable = self.stack.pop().expect("Expected fill");
        let v = match (self.resolve(&fill), self.resolve(&n)) {
            (x, &Variable::F64(n)) => {
                Variable::Array(Arc::new(vec![x.clone(); n as usize]))
            }
            _ => return Err(module.error(array_fill.n.source_range(),
                &format!("{}\nExpected number for length in `[value; length]`",
                    self.stack_trace())))
        };
        self.stack.push(v);
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
                        _ => return Err(module.error(right.source_range(),
                                    &format!("{}\nExpected something from the right side",
                                        self.stack_trace())))
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
                            _ => return Err(module.error(left.source_range(),
                                    &format!("{}\nExpected something from the left side",
                                        self.stack_trace())))
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
                _ => return Err(module.error(left.source_range(),
                                &format!("{}\nExpected item",
                                    self.stack_trace())))
            }
        } else {
            // Evaluate right side before left because the left leaves
            // an raw pointer on the stack which might point to wrong place
            // if there are side effects of the right side affecting it.
            match try!(self.expression(right, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(right.source_range(),
                        &format!("{}\nExpected something from the right side",
                            self.stack_trace())))
            };
            match try!(self.expression(left, Side::LeftInsert(false), module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(left.source_range(),
                        &format!("{}\nExpected something from the left side",
                            self.stack_trace())))
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
                                            return Err(module.error(
                                                left.source_range(),
                                                &format!("{}\nReturn has no value",
                                                    self.stack_trace())))
                                        }
                                    }
                                    _ => return Err(module.error(
                                            left.source_range(),
                                            &format!("{}\nExpected assigning to a number",
                                                self.stack_trace())))
                                };
                            }
                        }
                        &Variable::Vec4(b) => {
                            unsafe {
                                match *r {
                                    Variable::Vec4(ref mut n) => {
                                        match op {
                                            Set => *n = b,
                                            Add => *n = [n[0] + b[0], n[1] + b[1],
                                                         n[2] + b[2], n[3] + b[3]],
                                            Sub => *n = [n[0] - b[0], n[1] - b[1],
                                                         n[2] - b[2], n[3] - b[3]],
                                            Mul => *n = [n[0] * b[0], n[1] * b[1],
                                                         n[2] * b[2], n[3] * b[3]],
                                            Div => *n = [n[0] / b[0], n[1] / b[1],
                                                         n[2] / b[2], n[3] / b[3]],
                                            Rem => *n = [n[0] % b[0], n[1] % b[1],
                                                         n[2] % b[2], n[3] % b[3]],
                                            Pow => *n = [n[0].powf(b[0]), n[1].powf(b[1]),
                                                         n[2].powf(b[2]), n[3].powf(b[3])],
                                            Assign => {}
                                        }
                                    }
                                    Variable::Return => {
                                        if let Set = op {
                                            *r = Variable::Vec4(b)
                                        } else {
                                            return Err(module.error(
                                                left.source_range(),
                                                &format!("{}\nReturn has no value",
                                                    self.stack_trace())))
                                        }
                                    }
                                    _ => return Err(module.error(
                                            left.source_range(),
                                            &format!("{}\nExpected assigning to a vec4",
                                                self.stack_trace())))
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
                                            return Err(module.error(
                                                left.source_range(),
                                                &format!("{}\nReturn has no value",
                                                    self.stack_trace())))
                                        }
                                    }
                                    _ => return Err(module.error(
                                            left.source_range(),
                                            &format!("{}\nExpected assigning to a bool",
                                                self.stack_trace())))
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
                                            return Err(module.error(
                                                left.source_range(),
                                                &format!("{}\nReturn has no value",
                                                    self.stack_trace())))
                                        }
                                    }
                                    _ => return Err(module.error(
                                        left.source_range(),
                                        &format!("{}\nExpected assigning to text",
                                            self.stack_trace())))
                                }
                            }
                        }
                        &Variable::Object(ref b) => {
                            unsafe {
                                match *r {
                                    Variable::Object(ref mut n) => {
                                        if let Set = op {
                                            // Check address to avoid unsafe
                                            // reading and writing to same memory.
                                            let n_addr = n as *const _ as usize;
                                            let b_addr = b as *const _ as usize;
                                            if n_addr != b_addr {
                                                *r = Variable::Object(b.clone())
                                            }
                                            // *n = obj.clone()
                                        } else {
                                            unimplemented!()
                                        }
                                    }
                                    Variable::Return => {
                                        if let Set = op {
                                            *r = Variable::Object(b.clone())
                                        } else {
                                            return Err(module.error(
                                                left.source_range(),
                                                &format!("{}\nReturn has no value",
                                                    self.stack_trace())))
                                        }
                                    }
                                    _ => return Err(module.error(
                                        left.source_range(),
                                        &format!("{}\nExpected assigning to object",
                                            self.stack_trace())))
                                }
                            }
                        }
                        &Variable::Array(ref b) => {
                            unsafe {
                                match *r {
                                    Variable::Array(ref mut n) => {
                                        if let Set = op {
                                            // Check address to avoid unsafe
                                            // reading and writing to same memory.
                                            let n_addr = n as *const _ as usize;
                                            let b_addr = b as *const _ as usize;
                                            if n_addr != b_addr {
                                                *r = Variable::Array(b.clone())
                                            }
                                            // *n = arr.clone();
                                        } else {
                                            unimplemented!()
                                        }
                                    }
                                    Variable::Return => {
                                        if let Set = op {
                                            *r = Variable::Array(b.clone())
                                        } else {
                                            return Err(module.error(
                                                left.source_range(),
                                                &format!("{}\nReturn has no value",
                                                    self.stack_trace())))
                                        }
                                    }
                                    _ => return Err(module.error(
                                        left.source_range(),
                                        &format!("{}\nExpected assigning to array",
                                            self.stack_trace())))
                                }
                            }
                        }
                        &Variable::Option(ref b) => {
                            unsafe {
                                match *r {
                                    Variable::Option(ref mut n) => {
                                        if let Set = op {
                                            // Check address to avoid unsafe
                                            // reading and writing to same memory.
                                            let n_addr = n as *const _ as usize;
                                            let b_addr = b as *const _ as usize;
                                            if n_addr != b_addr {
                                                *r = Variable::Option(b.clone())
                                            }
                                        } else {
                                            unimplemented!()
                                        }
                                    }
                                    Variable::Return => {
                                        if let Set = op {
                                            *r = Variable::Option(b.clone())
                                        } else {
                                            return Err(module.error(
                                                left.source_range(),
                                                &format!("{}\nReturn has no value",
                                                    self.stack_trace())))
                                        }
                                    }
                                    _ => return Err(module.error(
                                        left.source_range(),
                                        &format!("{}\nExpected assigning to option",
                                            self.stack_trace())))
                                }
                            }
                        }
                        &Variable::Result(ref b) => {
                            unsafe {
                                match *r {
                                    Variable::Result(ref mut n) => {
                                        if let Set = op {
                                            // Check address to avoid unsafe
                                            // reading and writing to same memory.
                                            let n_addr = n as *const _ as usize;
                                            let b_addr = b as *const _ as usize;
                                            if n_addr != b_addr {
                                                *r = Variable::Result(b.clone())
                                            }
                                        } else {
                                            unimplemented!()
                                        }
                                    }
                                    Variable::Return => {
                                        if let Set = op {
                                            *r = Variable::Result(b.clone())
                                        } else {
                                            return Err(module.error(
                                                left.source_range(),
                                                &format!("{}\nReturn has no value",
                                                    self.stack_trace())))
                                        }
                                    }
                                    _ => return Err(module.error(
                                        left.source_range(),
                                        &format!("{}\nExpected assigning to result",
                                            self.stack_trace())))
                                }
                            }
                        }
                        &Variable::RustObject(ref b) => {
                            unsafe {
                                match *r {
                                    Variable::RustObject(ref mut n) => {
                                        if let Set = op {
                                            // Check address to avoid unsafe
                                            // reading and writing to same memory.
                                            let n_addr = n as *const _ as usize;
                                            let b_addr = b as *const _ as usize;
                                            if n_addr != b_addr {
                                                *r = Variable::RustObject(b.clone())
                                            }
                                        } else {
                                            unimplemented!()
                                        }
                                    }
                                    Variable::Return => {
                                        if let Set = op {
                                            *r = Variable::RustObject(b.clone())
                                        } else {
                                            return Err(module.error(
                                                left.source_range(),
                                                &format!("{}\nReturn has no value",
                                                    self.stack_trace())))
                                        }
                                    }
                                    _ => return Err(module.error(
                                        left.source_range(),
                                        &format!(
                                            "{}\nExpected assigning to rust_object",
                                            self.stack_trace())))
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
        use Error;

        #[inline(always)]
        fn try(
            stack: &mut Vec<Variable>,
            call_stack: &Vec<Call>,
            v: Result<Box<Variable>, Box<Error>>,
            locals: usize,
            source_range: Range,
            module: &Module
        ) -> Result<Flow, String> {
            match v {
                Ok(ref ok) => {
                    stack.push((**ok).clone());
                    Ok(Flow::Continue)
                }
                Err(ref err) => {
                    let ind = stack.len() - locals;
                    if let Variable::Return = stack[ind] {}
                    else {
                        return Err(module.error(source_range,
                            &format!("{}\nRequires `->` on function `{}`",
                            stack_trace(call_stack),
                            &call_stack.last().unwrap().fn_name)));
                    }
                    let mut err = err.clone();
                    let call = call_stack.last().unwrap();
                    let file = match call.file {
                        None => "".into(),
                        Some(ref f) => format!(" ({})", f)
                    };
                    err.trace.push(module.error(
                        source_range,
                        &format!("In function `{}`{}", call.fn_name, file)));
                    stack[ind] = Variable::Result(Err(err));
                    Ok(Flow::Return)
                }
            }
        }

        use ast::Id;

        if item.ids.len() == 0 {
            let name: &str = &**item.name;
            let locals = self.local_stack.len() - self.call_stack.last().unwrap().local_len;
            for &(ref n, id) in self.local_stack.iter().rev().take(locals) {
                if &**n == name {
                    if item.try {
                        // Check for `err(_)` or unwrap when `?` follows item.
                        let v = match self.resolve(&self.stack[id]) {
                            &Variable::Result(ref res) => res.clone(),
                            &Variable::Option(ref opt) => {
                                match opt {
                                    &Some(ref some) => Ok(some.clone()),
                                    &None => Err(Box::new(Error {
                                        message: Variable::Text(Arc::new(
                                            "Expected `some(_)`, found `none()`"
                                            .into())),
                                        trace: vec![]
                                    }))
                                }
                            }
                            _ => {
                                return Err(module.error(item.source_range,
                                    &format!("{}\nExpected `ok(_)` or `err(_)`",
                                        self.stack_trace())));
                            }
                        };
                        return try(&mut self.stack, &self.call_stack, v, locals,
                                   item.source_range, module);
                    } else {
                        self.stack.push(Variable::Ref(id));
                        return Ok(Flow::Continue);
                    }
                }
            }
            if name == "return" {
                return Err(module.error(item.source_range, &format!(
                    "{}\nRequires `->` on function `{}`",
                    self.stack_trace(),
                    &self.call_stack.last().unwrap().fn_name)));
            } else {
                return Err(module.error(item.source_range, &format!(
                    "{}\nCould not find local variable `{}`",
                        self.stack_trace(), name)));
            }
        }

        // Pre-evalutate expressions for identity.
        let start_stack_len = self.stack.len();
        for id in &item.ids {
            if let &Id::Expression(ref expr) = id {
                match try!(self.expression(expr, Side::Right, module)) {
                    (_, Flow::Return) => { return Ok(Flow::Return); }
                    (Expect::Something, Flow::Continue) => {}
                    _ => return Err(module.error(expr.source_range(),
                        &format!("{}\nExpected something for index",
                            self.stack_trace())))
                };
            }
        }
        let &mut Runtime {
            ref mut stack,
            ref mut local_stack,
            ref mut call_stack,
            ..
        } = self;
        let locals = local_stack.len() - call_stack.last().unwrap().local_len;
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
                let mut var: *mut Variable = try!(item_lookup(
                    module,
                    &mut stack[id],
                    stack,
                    call_stack,
                    &item.ids[0],
                    start_stack_len,
                    &mut expr_j,
                    insert,
                    item_len == 1
                ));
                let mut try_id_ind = 0;
                if item.try_ids.len() > 0 && item.try_ids[try_id_ind] == 0 {
                    // Check for error on `?` for first id.
                    let v = unsafe {match *var {
                        Variable::Result(ref res) => res.clone(),
                        Variable::Option(ref opt) => {
                            match opt {
                                &Some(ref some) => Ok(some.clone()),
                                &None => Err(Box::new(Error {
                                    message: Variable::Text(Arc::new(
                                        "Expected `some(_)`, found `none()`"
                                        .into())),
                                    trace: vec![]
                                }))
                            }
                        }
                        _ => {
                            return Err(module.error(item.ids[0].source_range(),
                                &format!("{}\nExpected `ok(_)` or `err(_)`",
                                    stack_trace(call_stack))));
                        }
                    }};
                    match v {
                        Ok(ref ok) => unsafe {
                            *var = (**ok).clone();
                            try_id_ind += 1;
                        },
                        Err(ref err) => {
                            let ind = stack.len() - locals;
                            if let Variable::Return = stack[ind] {}
                            else {
                                return Err(module.error(
                                    item.ids[0].source_range(),
                                    &format!("{}\nRequires `->` on function `{}`",
                                    stack_trace(call_stack),
                                    &call_stack.last().unwrap().fn_name)));
                            }
                            let mut err = err.clone();
                            let call = call_stack.last().unwrap();
                            let file = match call.file.as_ref() {
                                None => "".into(),
                                Some(f) => format!(" ({})", f)
                            };
                            err.trace.push(module.error(
                                item.ids[0].source_range(),
                                &format!("In function `{}`{}",
                                    &call.fn_name, file)));
                            stack[ind] = Variable::Result(Err(err));
                            return Ok(Flow::Return);
                        }
                    }
                }
                // Get the rest of the variables.
                for (i, prop) in item.ids[1..].iter().enumerate() {
                    var = try!(item_lookup(
                        module,
                        unsafe { &mut *var },
                        stack,
                        call_stack,
                        prop,
                        start_stack_len,
                        &mut expr_j,
                        insert,
                        // `i` skips first index.
                        i + 2 == item_len
                    ));

                    if item.try_ids.len() > try_id_ind &&
                       item.try_ids[try_id_ind] == i + 1 {
                        // Check for error on `?` for rest of ids.
                        let v = unsafe {match *var {
                            Variable::Result(ref res) => res.clone(),
                            Variable::Option(ref opt) => {
                                match opt {
                                    &Some(ref some) => Ok(some.clone()),
                                    &None => Err(Box::new(Error {
                                        message: Variable::Text(Arc::new(
                                            "Expected `some(_)`, found `none()`"
                                            .into())),
                                        trace: vec![]
                                    }))
                                }
                            }
                            _ => {
                                return Err(module.error(prop.source_range(),
                                    &format!("{}\nExpected `ok(_)` or `err(_)`",
                                        stack_trace(call_stack))));
                            }
                        }};
                        match v {
                            Ok(ref ok) => unsafe {
                                *var = (**ok).clone();
                                try_id_ind += 1;
                            },
                            Err(ref err) => {
                                let ind = stack.len() - locals;
                                if let Variable::Return = stack[ind] {}
                                else {
                                    return Err(module.error(
                                        prop.source_range(),
                                        &format!("{}\nRequires `->` on function `{}`",
                                            stack_trace(call_stack),
                                            &call_stack.last().unwrap().fn_name)));
                                }
                                let mut err = err.clone();
                                let call = call_stack.last().unwrap();
                                let file = match call.file.as_ref() {
                                    None => "".into(),
                                    Some(f) => format!(" ({})", f)
                                };
                                err.trace.push(module.error(
                                    prop.source_range(),
                                    &format!("In function `{}`{}",
                                        &call.fn_name, file)));
                                stack[ind] = Variable::Result(Err(err));
                                return Ok(Flow::Return);
                            }
                        }
                    }
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

    pub fn typeof_var(&self, var: &Variable) -> Arc<String> {
        let v = match var {
            &Variable::Text(_) => self.text_type.clone(),
            &Variable::F64(_) => self.f64_type.clone(),
            &Variable::Vec4(_) => self.vec4_type.clone(),
            &Variable::Return => self.return_type.clone(),
            &Variable::Bool(_) => self.bool_type.clone(),
            &Variable::Object(_) => self.object_type.clone(),
            &Variable::Array(_) => self.array_type.clone(),
            &Variable::Ref(_) => self.ref_type.clone(),
            &Variable::UnsafeRef(_) => self.unsafe_ref_type.clone(),
            &Variable::RustObject(_) => self.rust_object_type.clone(),
            &Variable::Option(_) => self.option_type.clone(),
            &Variable::Result(_) => self.result_type.clone(),
        };
        match v {
            Variable::Text(v) => v,
            _ => panic!("Expected string")
        }
    }

    fn compare(
        &mut self,
        compare: &ast::Compare,
        module: &Module
    ) -> Result<Flow, String> {
        fn sub_compare(
            rt: &Runtime,
            compare: &ast::Compare,
            module: &Module,
            a: &Variable,
            b: &Variable
        ) -> Result<bool, String> {
            use ast::CompareOp::*;

            match (rt.resolve(&b), rt.resolve(&a)) {
                (&Variable::F64(b), &Variable::F64(a)) => {
                    Ok(match compare.op {
                        Less => a < b,
                        LessOrEqual => a <= b,
                        Greater => a > b,
                        GreaterOrEqual => a >= b,
                        Equal => a == b,
                        NotEqual => a != b
                    })
                }
                (&Variable::Text(ref b), &Variable::Text(ref a)) => {
                    Ok(match compare.op {
                        Less => a < b,
                        LessOrEqual => a <= b,
                        Greater => a > b,
                        GreaterOrEqual => a >= b,
                        Equal => a == b,
                        NotEqual => a != b
                    })
                }
                (&Variable::Bool(b), &Variable::Bool(a)) => {
                    Ok(match compare.op {
                        Equal => a == b,
                        NotEqual => a != b,
                        x => return Err(module.error(compare.source_range,
                            &format!("{}\n`{}` can not be used with bools",
                                rt.stack_trace(),
                                x.symbol())))
                    })
                }
                (&Variable::Object(ref b), &Variable::Object(ref a)) => {
                    Ok(match compare.op {
                        Equal => a == b,
                        NotEqual => a != b,
                        x => return Err(module.error(compare.source_range,
                            &format!("{}\n`{}` can not be used with objects",
                                rt.stack_trace(),
                                x.symbol())))
                    })
                }
                (&Variable::Array(ref b), &Variable::Array(ref a)) => {
                    Ok(match compare.op {
                        Equal => a == b,
                        NotEqual => a != b,
                        x => return Err(module.error(compare.source_range,
                            &format!("{}\n`{}` can not be used with arrays",
                                rt.stack_trace(),
                                x.symbol())))
                    })
                }
                (&Variable::Option(None), &Variable::Option(None)) => {
                    Ok(match compare.op {
                        Equal => true,
                        NotEqual => false,
                        x => return Err(module.error(compare.source_range,
                            &format!("{}\n`{}` can not be used with options",
                                rt.stack_trace(),
                                x.symbol())))
                    })
                }
                (&Variable::Option(None), &Variable::Option(_)) => {
                    Ok(match compare.op {
                        Equal => false,
                        NotEqual => true,
                        x => return Err(module.error(compare.source_range,
                            &format!("{}\n`{}` can not be used with options",
                                rt.stack_trace(),
                                x.symbol())))
                    })
                }
                (&Variable::Option(_), &Variable::Option(None)) => {
                    Ok(match compare.op {
                        Equal => false,
                        NotEqual => true,
                        x => return Err(module.error(compare.source_range,
                            &format!("{}\n`{}` can not be used with options",
                                rt.stack_trace(),
                                x.symbol())))
                    })
                }
                (&Variable::Option(Some(ref b)),
                 &Variable::Option(Some(ref a))) => {
                    sub_compare(rt, compare, module, a, b)
                }
                (b, a) => return Err(module.error(compare.source_range,
                    &format!(
                    "{}\n`{}` can not be used with `{}` and `{}`",
                    rt.stack_trace(),
                    compare.op.symbol(),
                    rt.typeof_var(a),
                    rt.typeof_var(b))))
            }
        }

        match try!(self.expression(&compare.left, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(compare.left.source_range(),
                &format!("{}\nExpected something from the left argument",
                    self.stack_trace())))
        };
        match try!(self.expression(&compare.right, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(compare.right.source_range(),
                &format!("{}\nExpected something from the right argument",
                    self.stack_trace())))
        };
        match (self.stack.pop(), self.stack.pop()) {
            (Some(b), Some(a)) => {
                let v = try!(sub_compare(self, compare, module, &a, &b));
                self.stack.push(Variable::Bool(v))
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
            _ => return Err(module.error(if_expr.cond.source_range(),
                &format!("{}\nExpected bool from if condition",
                    self.stack_trace())))
        };
        let cond = self.stack.pop().expect("Expected bool");
        let val = match self.resolve(&cond) {
            &Variable::Bool(val) => val,
            _ => return Err(module.error(if_expr.cond.source_range(),
                &format!("{}\nExpected bool from if condition",
                    self.stack_trace())))
        };
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
                _ => return Err(module.error(cond.source_range(),
                    &format!("{}\nExpected bool from else if condition",
                        self.stack_trace())))
            };
            let else_if_cond = self.stack.pop().expect("Expected bool");
            match self.resolve(&else_if_cond) {
                &Variable::Bool(false) => {}
                &Variable::Bool(true) => {
                    return self.block(body, module);
                }
                _ => return Err(module.error(cond.source_range(),
                    &format!("{}\nExpected bool from else if condition",
                        self.stack_trace())))
            }
        }
        if let Some(ref block) = if_expr.else_block {
            self.block(block, module)
        } else {
            Ok((Expect::Nothing, Flow::Continue))
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
            _ => return Err(module.error(for_expr.init.source_range(),
                &format!("{}\nExpected nothing from for init",
                    self.stack_trace())))
        };
        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match try!(self.expression(&for_expr.cond, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(for_expr.cond.source_range(),
                    &format!("{}\nExpected bool from for condition",
                        self.stack_trace())))
            };
            match self.stack.pop() {
                None => panic!("There is no value on the stack"),
                Some(x) => {
                    let val = match x {
                        Variable::Bool(val) => val,
                        _ => return Err(module.error(
                            for_expr.cond.source_range(),
                            &format!("{}\nExpected bool", self.stack_trace())))
                    };
                    if !val { break }
                    match try!(self.block(&for_expr.block, module)) {
                        (_, Flow::Return) => { return Ok(Flow::Return); }
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
                                    _ => return Err(module.error(
                                        for_expr.step.source_range(),
                                        &format!("{}\nExpected nothing from for step",
                                            self.stack_trace())))
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
                            _ => return Err(module.error(
                                for_expr.step.source_range(),
                                &format!("{}\nExpected nothing from for step",
                                    self.stack_trace())))
                    };
                }
            };
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        Ok(flow)
    }
    fn for_n_expr(
        &mut self,
        for_n_expr: &ast::ForN,
        module: &Module
    ) -> Result<Flow, String> {
        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();

        let start = if let Some(ref start) = for_n_expr.start {
            // Evaluate start such that it's on the stack.
            match try!(self.expression(start, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(for_n_expr.end.source_range(),
                    &format!("{}\nExpected number from for start",
                        self.stack_trace())))
            };
            let start = self.stack.pop().expect("There is no value on the stack");
            let start = match self.resolve(&start) {
                &Variable::F64(val) => val,
                x => return Err(module.error(for_n_expr.end.source_range(),
                                &self.expected(x, "number")))
            };
            start
        } else { 0.0 };

        // Initialize counter.
        self.local_stack.push((for_n_expr.name.clone(), self.stack.len()));
        self.stack.push(Variable::F64(start));
        // Evaluate end such that it's on the stack.
        match try!(self.expression(&for_n_expr.end, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(for_n_expr.end.source_range(),
                &format!("{}\nExpected number from for end",
                    self.stack_trace())))
        };
        let end = self.stack.pop().expect("There is no value on the stack");
        let end = match self.resolve(&end) {
            &Variable::F64(val) => val,
            x => return Err(module.error(for_n_expr.end.source_range(),
                            &self.expected(x, "number")))
        };
        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match &self.stack[st - 1] {
                &Variable::F64(val) => {
                    if val < end {}
                    else { break }
                }
                x => return Err(module.error(for_n_expr.source_range,
                                &self.expected(x, "number")))
            };
            match try!(self.block(&for_n_expr.block, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (_, Flow::Continue) => {}
                (_, Flow::Break(x)) => {
                    match x {
                        Some(label) => {
                            let same =
                            if let Some(ref for_label) = for_n_expr.label {
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
                            if let Some(ref for_label) = for_n_expr.label {
                                &label == for_label
                            } else { false };
                            if !same {
                                flow = Flow::ContinueLoop(Some(label));
                                break;
                            }
                        }
                        None => {}
                    }
                }
            }
            let error = if let Variable::F64(ref mut val) = self.stack[st - 1] {
                *val += 1.0;
                false
            } else { true };
            if error {
                return Err(module.error(for_n_expr.source_range,
                           &self.expected(&self.stack[st - 1], "number")))
            }
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        Ok(flow)
    }
    fn sum_n_expr(
        &mut self,
        for_n_expr: &ast::ForN,
        module: &Module
    ) -> Result<Flow, String> {
        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();
        let mut sum = 0.0;

        let start = if let Some(ref start) = for_n_expr.start {
            // Evaluate start such that it's on the stack.
            match try!(self.expression(start, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(for_n_expr.end.source_range(),
                    &format!("{}\nExpected number from for start",
                        self.stack_trace())))
            };
            let start = self.stack.pop().expect("There is no value on the stack");
            let start = match self.resolve(&start) {
                &Variable::F64(val) => val,
                x => return Err(module.error(for_n_expr.end.source_range(),
                                &self.expected(x, "number")))
            };
            start
        } else { 0.0 };

        // Initialize counter.
        self.local_stack.push((for_n_expr.name.clone(), self.stack.len()));
        self.stack.push(Variable::F64(start));
        // Evaluate end such that it's on the stack.
        match try!(self.expression(&for_n_expr.end, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(for_n_expr.end.source_range(),
                &format!("{}\nExpected number from for end",
                    self.stack_trace())))
        };
        let end = self.stack.pop().expect("There is no value on the stack");
        let end = match self.resolve(&end) {
            &Variable::F64(val) => val,
            x => return Err(module.error(for_n_expr.end.source_range(),
                            &self.expected(x, "number")))
        };
        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match &self.stack[st - 1] {
                &Variable::F64(val) => {
                    if val < end {}
                    else { break }
                }
                x => return Err(module.error(for_n_expr.source_range,
                                &self.expected(x, "number")))
            };
            match try!(self.block(&for_n_expr.block, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {
                    match self.resolve(self.stack.last()
                              .expect("There is no value on the stack")) {
                        &Variable::F64(val) => sum += val,
                        x => return Err(module.error(for_n_expr.block.source_range,
                                &self.expected(x, "number")))
                    };
                }
                (Expect::Nothing, Flow::Continue) => {
                    return Err(module.error(for_n_expr.block.source_range,
                                "Expected `number`"))
                }
                (_, Flow::Break(x)) => {
                    match x {
                        Some(label) => {
                            let same =
                            if let Some(ref for_label) = for_n_expr.label {
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
                            if let Some(ref for_label) = for_n_expr.label {
                                &label == for_label
                            } else { false };
                            if !same {
                                flow = Flow::ContinueLoop(Some(label));
                                break;
                            }
                        }
                        None => {}
                    }
                }
            }
            let error = if let Variable::F64(ref mut val) = self.stack[st - 1] {
                *val += 1.0;
                false
            } else { true };
            if error {
                return Err(module.error(for_n_expr.source_range,
                           &self.expected(&self.stack[st - 1], "number")))
            }
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        self.stack.push(Variable::F64(sum));
        Ok(flow)
    }
    fn min_n_expr(
        &mut self,
        for_n_expr: &ast::ForN,
        module: &Module
    ) -> Result<Flow, String> {
        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();

        let start = if let Some(ref start) = for_n_expr.start {
            // Evaluate start such that it's on the stack.
            match try!(self.expression(start, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(for_n_expr.end.source_range(),
                    &format!("{}\nExpected number from for start",
                        self.stack_trace())))
            };
            let start = self.stack.pop().expect("There is no value on the stack");
            let start = match self.resolve(&start) {
                &Variable::F64(val) => val,
                x => return Err(module.error(for_n_expr.end.source_range(),
                                &self.expected(x, "number")))
            };
            start
        } else { 0.0 };

        let mut min: Option<(f64, f64)> = None;
        // Initialize counter.
        self.local_stack.push((for_n_expr.name.clone(), self.stack.len()));
        self.stack.push(Variable::F64(start));
        // Evaluate end such that it's on the stack.
        match try!(self.expression(&for_n_expr.end, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(for_n_expr.end.source_range(),
                &format!("{}\nExpected number from for end",
                    self.stack_trace())))
        };
        let end = self.stack.pop().expect("There is no value on the stack");
        let end = match self.resolve(&end) {
            &Variable::F64(val) => val,
            x => return Err(module.error(for_n_expr.end.source_range(),
                            &self.expected(x, "number")))
        };
        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            let ind = match &self.stack[st - 1] {
                &Variable::F64(val) => {
                    if val < end {}
                    else { break }
                    val
                }
                x => return Err(module.error(for_n_expr.source_range,
                                &self.expected(x, "number")))
            };
            match try!(self.block(&for_n_expr.block, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {
                    match self.resolve(self.stack.last()
                              .expect("There is no value on the stack")) {
                        &Variable::F64(val) => {
                            if let Some((ref mut min_arg, ref mut min_val)) = min {
                                if *min_val > val {
                                    *min_arg = ind;
                                    *min_val = val;
                                }
                            } else {
                                min = Some((ind, val));
                            }
                        },
                        x => return Err(module.error(for_n_expr.block.source_range,
                                &self.expected(x, "number")))
                    };
                }
                (Expect::Nothing, Flow::Continue) => {
                    return Err(module.error(for_n_expr.block.source_range,
                                "Expected `number`"))
                }
                (_, Flow::Break(x)) => {
                    match x {
                        Some(label) => {
                            let same =
                            if let Some(ref for_label) = for_n_expr.label {
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
                            if let Some(ref for_label) = for_n_expr.label {
                                &label == for_label
                            } else { false };
                            if !same {
                                flow = Flow::ContinueLoop(Some(label));
                                break;
                            }
                        }
                        None => {}
                    }
                }
            }
            let error = if let Variable::F64(ref mut val) = self.stack[st - 1] {
                *val += 1.0;
                false
            } else { true };
            if error {
                return Err(module.error(for_n_expr.source_range,
                           &self.expected(&self.stack[st - 1], "number")))
            }
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        self.stack.push(match min {
            None => Variable::Option(None),
            Some((arg, val)) => Variable::Option(Some(Box::new(
                Variable::Array(Arc::new(vec![Variable::F64(arg), Variable::F64(val)]))
            )))
        });
        Ok(flow)
    }
    fn max_n_expr(
        &mut self,
        for_n_expr: &ast::ForN,
        module: &Module
    ) -> Result<Flow, String> {
        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();

        let start = if let Some(ref start) = for_n_expr.start {
            // Evaluate start such that it's on the stack.
            match try!(self.expression(start, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(for_n_expr.end.source_range(),
                    &format!("{}\nExpected number from for start",
                        self.stack_trace())))
            };
            let start = self.stack.pop().expect("There is no value on the stack");
            let start = match self.resolve(&start) {
                &Variable::F64(val) => val,
                x => return Err(module.error(for_n_expr.end.source_range(),
                                &self.expected(x, "number")))
            };
            start
        } else { 0.0 };

        let mut max: Option<(f64, f64)> = None;
        // Initialize counter.
        self.local_stack.push((for_n_expr.name.clone(), self.stack.len()));
        self.stack.push(Variable::F64(start));
        // Evaluate end such that it's on the stack.
        match try!(self.expression(&for_n_expr.end, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(for_n_expr.end.source_range(),
                &format!("{}\nExpected number from for end",
                    self.stack_trace())))
        };
        let end = self.stack.pop().expect("There is no value on the stack");
        let end = match self.resolve(&end) {
            &Variable::F64(val) => val,
            x => return Err(module.error(for_n_expr.end.source_range(),
                            &self.expected(x, "number")))
        };
        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            let ind = match &self.stack[st - 1] {
                &Variable::F64(val) => {
                    if val < end {}
                    else { break }
                    val
                }
                x => return Err(module.error(for_n_expr.source_range,
                                &self.expected(x, "number")))
            };
            match try!(self.block(&for_n_expr.block, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {
                    match self.resolve(self.stack.last()
                              .expect("There is no value on the stack")) {
                        &Variable::F64(val) => {
                            if let Some((ref mut max_arg, ref mut max_val)) = max {
                                if *max_val < val {
                                    *max_arg = ind;
                                    *max_val = val;
                                }
                            } else {
                                max = Some((ind, val));
                            }
                        },
                        x => return Err(module.error(for_n_expr.block.source_range,
                                &self.expected(x, "number")))
                    };
                }
                (Expect::Nothing, Flow::Continue) => {
                    return Err(module.error(for_n_expr.block.source_range,
                                "Expected `number`"))
                }
                (_, Flow::Break(x)) => {
                    match x {
                        Some(label) => {
                            let same =
                            if let Some(ref for_label) = for_n_expr.label {
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
                            if let Some(ref for_label) = for_n_expr.label {
                                &label == for_label
                            } else { false };
                            if !same {
                                flow = Flow::ContinueLoop(Some(label));
                                break;
                            }
                        }
                        None => {}
                    }
                }
            }
            let error = if let Variable::F64(ref mut val) = self.stack[st - 1] {
                *val += 1.0;
                false
            } else { true };
            if error {
                return Err(module.error(for_n_expr.source_range,
                           &self.expected(&self.stack[st - 1], "number")))
            }
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        self.stack.push(match max {
            None => Variable::Option(None),
            Some((arg, val)) => Variable::Option(Some(Box::new(
                Variable::Array(Arc::new(vec![Variable::F64(arg), Variable::F64(val)]))
            )))
        });
        Ok(flow)
    }
    fn any_n_expr(
        &mut self,
        for_n_expr: &ast::ForN,
        module: &Module
    ) -> Result<Flow, String> {
        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();

        let start = if let Some(ref start) = for_n_expr.start {
            // Evaluate start such that it's on the stack.
            match try!(self.expression(start, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(for_n_expr.end.source_range(),
                    &format!("{}\nExpected number from for start",
                        self.stack_trace())))
            };
            let start = self.stack.pop().expect("There is no value on the stack");
            let start = match self.resolve(&start) {
                &Variable::F64(val) => val,
                x => return Err(module.error(for_n_expr.end.source_range(),
                                &self.expected(x, "number")))
            };
            start
        } else { 0.0 };

        let mut any = false;
        // Initialize counter.
        self.local_stack.push((for_n_expr.name.clone(), self.stack.len()));
        self.stack.push(Variable::F64(start));
        // Evaluate end such that it's on the stack.
        match try!(self.expression(&for_n_expr.end, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(for_n_expr.end.source_range(),
                &format!("{}\nExpected number from for end",
                    self.stack_trace())))
        };
        let end = self.stack.pop().expect("There is no value on the stack");
        let end = match self.resolve(&end) {
            &Variable::F64(val) => val,
            x => return Err(module.error(for_n_expr.end.source_range(),
                            &self.expected(x, "number")))
        };
        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match &self.stack[st - 1] {
                &Variable::F64(val) => {
                    if val < end {}
                    else { break }
                }
                x => return Err(module.error(for_n_expr.source_range,
                                &self.expected(x, "number")))
            };
            match try!(self.block(&for_n_expr.block, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {
                    match self.resolve(self.stack.last()
                              .expect("There is no value on the stack")) {
                        &Variable::Bool(val) => {
                            if val {
                                any = true;
                                break;
                            }
                        },
                        x => return Err(module.error(for_n_expr.block.source_range,
                                &self.expected(x, "booleal")))
                    };
                }
                (Expect::Nothing, Flow::Continue) => {
                    return Err(module.error(for_n_expr.block.source_range,
                                "Expected `booleal`"))
                }
                (_, Flow::Break(x)) => {
                    match x {
                        Some(label) => {
                            let same =
                            if let Some(ref for_label) = for_n_expr.label {
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
                            if let Some(ref for_label) = for_n_expr.label {
                                &label == for_label
                            } else { false };
                            if !same {
                                flow = Flow::ContinueLoop(Some(label));
                                break;
                            }
                        }
                        None => {}
                    }
                }
            }
            let error = if let Variable::F64(ref mut val) = self.stack[st - 1] {
                *val += 1.0;
                false
            } else { true };
            if error {
                return Err(module.error(for_n_expr.source_range,
                           &self.expected(&self.stack[st - 1], "number")))
            }
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        self.stack.push(Variable::Bool(any));
        Ok(flow)
    }
    fn all_n_expr(
        &mut self,
        for_n_expr: &ast::ForN,
        module: &Module
    ) -> Result<Flow, String> {
        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();

        let start = if let Some(ref start) = for_n_expr.start {
            // Evaluate start such that it's on the stack.
            match try!(self.expression(start, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(for_n_expr.end.source_range(),
                    &format!("{}\nExpected number from for start",
                        self.stack_trace())))
            };
            let start = self.stack.pop().expect("There is no value on the stack");
            let start = match self.resolve(&start) {
                &Variable::F64(val) => val,
                x => return Err(module.error(for_n_expr.end.source_range(),
                                &self.expected(x, "number")))
            };
            start
        } else { 0.0 };

        let mut any = true;
        // Initialize counter.
        self.local_stack.push((for_n_expr.name.clone(), self.stack.len()));
        self.stack.push(Variable::F64(start));
        // Evaluate end such that it's on the stack.
        match try!(self.expression(&for_n_expr.end, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(for_n_expr.end.source_range(),
                &format!("{}\nExpected number from for end",
                    self.stack_trace())))
        };
        let end = self.stack.pop().expect("There is no value on the stack");
        let end = match self.resolve(&end) {
            &Variable::F64(val) => val,
            x => return Err(module.error(for_n_expr.end.source_range(),
                            &self.expected(x, "number")))
        };
        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match &self.stack[st - 1] {
                &Variable::F64(val) => {
                    if val < end {}
                    else { break }
                }
                x => return Err(module.error(for_n_expr.source_range,
                                &self.expected(x, "number")))
            };
            match try!(self.block(&for_n_expr.block, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {
                    match self.resolve(self.stack.last()
                              .expect("There is no value on the stack")) {
                        &Variable::Bool(val) => {
                            if !val {
                                any = false;
                                break;
                            }
                        },
                        x => return Err(module.error(for_n_expr.block.source_range,
                                &self.expected(x, "booleal")))
                    };
                }
                (Expect::Nothing, Flow::Continue) => {
                    return Err(module.error(for_n_expr.block.source_range,
                                "Expected `booleal`"))
                }
                (_, Flow::Break(x)) => {
                    match x {
                        Some(label) => {
                            let same =
                            if let Some(ref for_label) = for_n_expr.label {
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
                            if let Some(ref for_label) = for_n_expr.label {
                                &label == for_label
                            } else { false };
                            if !same {
                                flow = Flow::ContinueLoop(Some(label));
                                break;
                            }
                        }
                        None => {}
                    }
                }
            }
            let error = if let Variable::F64(ref mut val) = self.stack[st - 1] {
                *val += 1.0;
                false
            } else { true };
            if error {
                return Err(module.error(for_n_expr.source_range,
                           &self.expected(&self.stack[st - 1], "number")))
            }
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        self.stack.push(Variable::Bool(any));
        Ok(flow)
    }
    fn sift_n_expr(
        &mut self,
        for_n_expr: &ast::ForN,
        module: &Module
    ) -> Result<Flow, String> {
        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();
        let mut res: Vec<Variable> = vec![];

        let start = if let Some(ref start) = for_n_expr.start {
            // Evaluate start such that it's on the stack.
            match try!(self.expression(start, Side::Right, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(for_n_expr.end.source_range(),
                    &format!("{}\nExpected number from for start",
                        self.stack_trace())))
            };
            let start = self.stack.pop().expect("There is no value on the stack");
            let start = match self.resolve(&start) {
                &Variable::F64(val) => val,
                x => return Err(module.error(for_n_expr.end.source_range(),
                                &self.expected(x, "number")))
            };
            start
        } else { 0.0 };

        // Initialize counter.
        self.local_stack.push((for_n_expr.name.clone(), self.stack.len()));
        self.stack.push(Variable::F64(start));

        // Evaluate end such that it's on the stack.
        match try!(self.expression(&for_n_expr.end, Side::Right, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(for_n_expr.end.source_range(),
                &format!("{}\nExpected number from for end",
                    self.stack_trace())))
        };
        let end = self.stack.pop().expect("There is no value on the stack");
        let end = match self.resolve(&end) {
            &Variable::F64(val) => val,
            x => return Err(module.error(for_n_expr.end.source_range(),
                            &self.expected(x, "number")))
        };
        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match &self.stack[st - 1] {
                &Variable::F64(val) => {
                    if val < end {}
                    else { break }
                }
                x => return Err(module.error(for_n_expr.source_range,
                                &self.expected(x, "number")))
            };
            match try!(self.block(&for_n_expr.block, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {
                    res.push(self.stack.pop()
                       .expect("There is no value on the stack"));
                }
                (Expect::Nothing, Flow::Continue) => {
                    return Err(module.error(for_n_expr.block.source_range,
                                "Expected variable"))
                }
                (_, Flow::Break(x)) => {
                    match x {
                        Some(label) => {
                            let same =
                            if let Some(ref for_label) = for_n_expr.label {
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
                            if let Some(ref for_label) = for_n_expr.label {
                                &label == for_label
                            } else { false };
                            if !same {
                                flow = Flow::ContinueLoop(Some(label));
                                break;
                            }
                        }
                        None => {}
                    }
                }
            }
            let error = if let Variable::F64(ref mut val) = self.stack[st - 1] {
                *val += 1.0;
                false
            } else { true };
            if error {
                return Err(module.error(for_n_expr.source_range,
                           &self.expected(&self.stack[st - 1], "number")))
            }
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        self.stack.push(Variable::Array(Arc::new(res)));
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
    fn vec4(
        &mut self,
        vec4: &ast::Vec4,
        side: Side,
        module: &Module
    ) -> Result<Flow, String> {
        for expr in &vec4.args {
            match try!(self.expression(expr, side, module)) {
                (_, Flow::Return) => { return Ok(Flow::Return); }
                (Expect::Something, Flow::Continue) => {}
                _ => return Err(module.error(expr.source_range(),
                    &format!("{}\nExpected something from vec4 argument",
                        self.stack_trace())))
            };
        }
        let w = self.stack.pop().expect("There is no value on the stack");
        let w = match self.resolve(&w) {
            &Variable::F64(val) => val,
            x => return Err(module.error(vec4.args[3].source_range(),
                &self.expected(x, "number")))
        };
        let z = self.stack.pop().expect("There is no value on the stack");
        let z = match self.resolve(&z) {
            &Variable::F64(val) => val,
            x => return Err(module.error(vec4.args[2].source_range(),
                &self.expected(x, "number")))
        };
        let y = self.stack.pop().expect("There is no value on the stack");
        let y = match self.resolve(&y) {
            &Variable::F64(val) => val,
            x => return Err(module.error(vec4.args[1].source_range(),
                &self.expected(x, "number")))
        };
        let x = self.stack.pop().expect("There is no value on the stack");
        let x = match self.resolve(&x) {
            &Variable::F64(val) => val,
            x => return Err(module.error(vec4.args[0].source_range(),
                &self.expected(x, "number")))
        };
        self.stack.push(Variable::Vec4([x as f32, y as f32, z as f32, w as f32]));
        Ok(Flow::Continue)
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
            _ => return Err(module.error(unop.source_range,
                &format!("{}\nExpected something from unary argument",
                    self.stack_trace())))
        };
        let val = self.stack.pop().expect("Expected unary argument");
        let v = match self.resolve(&val) {
            &Variable::Vec4(b) => {
                Variable::F64(match unop.op {
                    ast::UnOp::Norm => (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt() as f64,
                    _ => return Err(module.error(unop.source_range,
                                    &format!("{}\nUnknown vec4 unary operator",
                                             self.stack_trace())))
                })
            }
            &Variable::Bool(b) => {
                Variable::Bool(match unop.op {
                    ast::UnOp::Not => !b,
                    _ => return Err(module.error(unop.source_range,
                                    &format!("{}\nUnknown boolean unary operator",
                                             self.stack_trace())))
                })
            }
            &Variable::F64(v) => {
                Variable::F64(match unop.op {
                    ast::UnOp::Neg => -v,
                    _ => return Err(module.error(unop.source_range,
                                    &format!("{}\nUnknown number unary operator",
                                             self.stack_trace())))
                })
            }
            _ => return Err(module.error(unop.source_range,
                &format!("{}\nInvalid type, expected bool", self.stack_trace())))
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
                &format!("{}\nExpected something from left argument",
                    self.stack_trace())))
        };
        match try!(self.expression(&binop.right, side, module)) {
            (_, Flow::Return) => { return Ok(Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => return Err(module.error(binop.source_range,
                &format!("{}\nExpected something from right argument",
                    self.stack_trace())))
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
                    Pow => a.powf(b),
                    _ => return Err(module.error(binop.source_range,
                        &format!("{}\nUnknown number operator `{:?}`",
                            self.stack_trace(),
                            binop.op.symbol())))
                })
            }
            (&Variable::Vec4(a), &Variable::Vec4(b)) => {
                match binop.op {
                    Add => Variable::Vec4([a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]),
                    Sub => Variable::Vec4([a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]),
                    Mul => Variable::Vec4([a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]),
                    Dot => Variable::F64((a[0] * b[0] + a[1] * b[1] +
                                          a[2] * b[2] + a[3] * b[3]) as f64),
                    Cross => Variable::Vec4([a[1] * b[2] - a[2] * b[1],
                                             a[2] * b[0] - a[0] * b[2],
                                             a[0] * b[1] - a[1] * b[0], 0.0]),
                    Div => Variable::Vec4([a[0] / b[0], a[1] / b[1], a[2] / b[2], a[3] / b[3]]),
                    Rem => Variable::Vec4([a[0] % b[0], a[1] % b[1], a[2] % b[2], a[3] % b[3]]),
                    Pow => Variable::Vec4([a[0].powf(b[0]), a[1].powf(b[1]),
                                           a[2].powf(b[2]), a[3].powf(b[3])])
                }
            }
            (&Variable::Vec4(a), &Variable::F64(b)) => {
                let b = b as f32;
                match binop.op {
                    Add => Variable::Vec4([a[0] + b, a[1] + b, a[2] + b, a[3] + b]),
                    Sub => Variable::Vec4([a[0] - b, a[1] - b, a[2] - b, a[3] - b]),
                    Mul => Variable::Vec4([a[0] * b, a[1] * b, a[2] * b, a[3] * b]),
                    Dot => Variable::F64((a[0] * b + a[1] * b +
                                          a[2] * b + a[3] * b) as f64),
                    Cross => return Err(module.error(binop.source_range,
                        &format!("{}\nExpected two vec4 for `{:?}`",
                            self.stack_trace(), binop.op.symbol()))),
                    Div => Variable::Vec4([a[0] / b, a[1] / b, a[2] / b, a[3] / b]),
                    Rem => Variable::Vec4([a[0] % b, a[1] % b, a[2] % b, a[3] % b]),
                    Pow => Variable::Vec4([a[0].powf(b), a[1].powf(b),
                                           a[2].powf(b), a[3].powf(b)]),
                }
            }
            (&Variable::F64(a), &Variable::Vec4(b)) => {
                let a = a as f32;
                match binop.op {
                    Add => Variable::Vec4([a + b[0], a + b[1], a + b[2], a + b[3]]),
                    Sub => Variable::Vec4([a - b[0], a - b[1], a - b[2], a - b[3]]),
                    Mul => Variable::Vec4([a * b[0], a * b[1], a * b[2], a * b[3]]),
                    Dot => Variable::F64((a * b[0] + a * b[1] +
                                          a * b[2] + a * b[3]) as f64),
                    Cross => return Err(module.error(binop.source_range,
                        &format!("{}\nExpected two vec4 for `{:?}`",
                            self.stack_trace(), binop.op.symbol()))),
                    Div => Variable::Vec4([a / b[0], a / b[1], a / b[2], a / b[3]]),
                    Rem => Variable::Vec4([a % b[0], a % b[1], a % b[2], a % b[3]]),
                    Pow => Variable::Vec4([a.powf(b[0]), a.powf(b[1]),
                                           a.powf(b[2]), a.powf(b[3])])
                }
            }
            (&Variable::Bool(a), &Variable::Bool(b)) => {
                Variable::Bool(match binop.op {
                    Add => a || b,
                    // Boolean subtraction with lazy precedence.
                    Sub => a && !b,
                    Mul => a && b,
                    Pow => a ^ b,
                    _ => return Err(module.error(binop.source_range,
                        &format!("{}\nUnknown boolean operator `{:?}`",
                            self.stack_trace(),
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
                        &format!("{}\nThis operation can not be used with strings",
                            self.stack_trace())))
                }
            }
            (&Variable::Text(_), _) =>
                return Err(module.error(binop.source_range,
                &format!("{}\nThe right argument must be a string. \
                Try the `to_string` function", self.stack_trace()))),
            _ => return Err(module.error(binop.source_range, &format!(
                "{}\nInvalid type for binary operator `{:?}`, \
                expected numbers, vec4s, bools or strings",
                self.stack_trace(),
                binop.op.symbol())))
        };
        self.stack.push(v);

        Ok(Flow::Continue)
    }
    pub fn stack_trace(&self) -> String {
        stack_trace(&self.call_stack)
    }
}

fn stack_trace(call_stack: &[Call]) -> String {
    let mut s = String::new();
    for call in call_stack.iter() {
        s.push_str(&call.fn_name);
        if let Some(ref file) = call.file {
            s.push_str(" (");
            s.push_str(file);
            s.push(')');
        }
        s.push('\n')
    }
    s
}
