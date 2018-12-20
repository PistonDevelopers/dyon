use super::*;

macro_rules! iter(
    ($rt:ident, $for_in_expr:ident, $module:ident) => {{
        let iter = match $rt.expression(&$for_in_expr.iter, Side::Right, $module)? {
            (x, Flow::Return) => { return Ok((x, Flow::Return)); }
            (Some(x), Flow::Continue) => x,
            _ => return Err($module.error($for_in_expr.iter.source_range(),
                &format!("{}\nExpected in-type from for iter",
                    $rt.stack_trace()), $rt))
        };
        match $rt.resolve(&iter) {
            &Variable::In(ref val) => val.clone(),
            x => return Err($module.error($for_in_expr.iter.source_range(),
                            &$rt.expected(x, "in"), $rt))
        }
    }};
);

macro_rules! iter_val(
    ($iter:ident, $rt:ident, $for_in_expr:ident, $module:ident) => {
        match $iter.lock() {
            Ok(x) => match x.try_recv() {
                Ok(x) => x,
                Err(_) => return Ok((None, Flow::Continue)),
            },
            Err(err) => {
                return Err($module.error($for_in_expr.source_range,
                &format!("Can not lock In mutex:\n{}", err.description()), $rt));
            }
        }
    };
);

macro_rules! break_(
    ($x:ident, $for_in_expr:ident, $flow:ident) => {{
        if let Some(label) = $x {
            let same =
            if let Some(ref for_label) = $for_in_expr.label {
                &label == for_label
            } else { false };
            if !same {
                $flow = Flow::Break(Some(label))
            }
        }
        break;
    }};
);

macro_rules! continue_(
    ($x:ident, $for_in_expr:ident, $flow:ident) => {
        if let Some(label) = $x {
            let same =
            if let Some(ref for_label) = $for_in_expr.label {
                &label == for_label
            } else { false };
            if !same {
                $flow = Flow::ContinueLoop(Some(label));
                break;
            }
        }
    };
);

macro_rules! iter_val_inc(
    ($iter:ident, $rt:ident, $for_in_expr:ident, $module:ident) => {
        match $iter.lock() {
            Ok(x) => match x.try_recv() {
                Ok(x) => x,
                Err(_) => break,
            },
            Err(err) => {
                return Err($module.error($for_in_expr.source_range,
                &format!("Can not lock In mutex:\n{}", err.description()), $rt));
            }
        }
    };
);

impl Runtime {
    pub(crate) fn for_in_expr(
        &mut self,
        for_in_expr: &ast::ForIn,
        module: &Arc<Module>
    ) -> Result<(Option<Variable>, Flow), String> {
        use std::error::Error;

        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();

        let iter = iter!(self, for_in_expr, module);
        let iter_val = iter_val!(iter, self, for_in_expr, module);

        // Initialize counter.
        self.local_stack.push((for_in_expr.name.clone(), self.stack.len()));
        self.stack.push(iter_val);

        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match self.block(&for_in_expr.block, module)? {
                (x, Flow::Return) => { return Ok((x, Flow::Return)); }
                (_, Flow::Continue) => {}
                (_, Flow::Break(x)) => break_!(x, for_in_expr, flow),
                (_, Flow::ContinueLoop(x)) => continue_!(x, for_in_expr, flow),
            }

            self.stack[st - 1] = iter_val_inc!(iter, self, for_in_expr, module);
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        Ok((None, flow))
    }

    pub(crate) fn sum_in_expr(
        &mut self,
        for_in_expr: &ast::ForIn,
        module: &Arc<Module>
    ) -> Result<(Option<Variable>, Flow), String> {
        use std::error::Error;

        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();

        let iter = iter!(self, for_in_expr, module);
        let iter_val = iter_val!(iter, self, for_in_expr, module);

        let mut sum = 0.0;

        // Initialize counter.
        self.local_stack.push((for_in_expr.name.clone(), self.stack.len()));
        self.stack.push(iter_val);

        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match self.block(&for_in_expr.block, module)? {
                (Some(x), Flow::Continue) => {
                    match self.resolve(&x) {
                        &Variable::F64(val, _) => sum += val,
                        x => return Err(module.error(for_in_expr.block.source_range,
                                &self.expected(x, "number"), self))
                    };
                }
                (x, Flow::Return) => { return Ok((x, Flow::Return)); }
                (_, Flow::Continue) => {}
                (_, Flow::Break(x)) => break_!(x, for_in_expr, flow),
                (_, Flow::ContinueLoop(x)) => continue_!(x, for_in_expr, flow),
            }

            self.stack[st - 1] = iter_val_inc!(iter, self, for_in_expr, module);
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        Ok((Some(Variable::f64(sum)), flow))
    }

    pub(crate) fn prod_in_expr(
        &mut self,
        for_in_expr: &ast::ForIn,
        module: &Arc<Module>
    ) -> Result<(Option<Variable>, Flow), String> {
        use std::error::Error;

        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();

        let iter = iter!(self, for_in_expr, module);
        let iter_val = iter_val!(iter, self, for_in_expr, module);

        let mut prod = 1.0;

        // Initialize counter.
        self.local_stack.push((for_in_expr.name.clone(), self.stack.len()));
        self.stack.push(iter_val);

        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match self.block(&for_in_expr.block, module)? {
                (Some(x), Flow::Continue) => {
                    match self.resolve(&x) {
                        &Variable::F64(val, _) => prod *= val,
                        x => return Err(module.error(for_in_expr.block.source_range,
                                &self.expected(x, "number"), self))
                    };
                }
                (x, Flow::Return) => { return Ok((x, Flow::Return)); }
                (_, Flow::Continue) => {}
                (_, Flow::Break(x)) => break_!(x, for_in_expr, flow),
                (_, Flow::ContinueLoop(x)) => continue_!(x, for_in_expr, flow),
            }

            self.stack[st - 1] = iter_val_inc!(iter, self, for_in_expr, module);
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        Ok((Some(Variable::f64(prod)), flow))
    }

    pub(crate) fn min_in_expr(
        &mut self,
        for_in_expr: &ast::ForIn,
        module: &Arc<Module>
    ) -> Result<(Option<Variable>, Flow), String> {
        use std::error::Error;

        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();

        let iter = iter!(self, for_in_expr, module);
        let iter_val = iter_val!(iter, self, for_in_expr, module);

        let mut min = ::std::f64::NAN;
        let mut sec = None;
        // Initialize counter.
        self.local_stack.push((for_in_expr.name.clone(), self.stack.len()));
        self.stack.push(iter_val);
        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match self.block(&for_in_expr.block, module)? {
                (Some(x), Flow::Continue) => {
                    match self.resolve(&x) {
                        &Variable::F64(val, ref val_sec) => {
                            if min.is_nan() || min > val {
                                min = val;
                                sec = match *val_sec {
                                    None => {
                                        Some(Box::new(vec![self.stack[st - 1].clone()]))
                                    }
                                    Some(ref arr) => {
                                        let mut arr = arr.clone();
                                        arr.push(self.stack[st - 1].clone());
                                        Some(arr)
                                    }
                                };
                            }
                        },
                        x => return Err(module.error(for_in_expr.block.source_range,
                                &self.expected(x, "number"), self))
                    };
                }
                (x, Flow::Return) => { return Ok((x, Flow::Return)); }
                (None, Flow::Continue) => {
                    return Err(module.error(for_in_expr.block.source_range,
                                "Expected `number or option`", self))
                }
                (_, Flow::Break(x)) => break_!(x, for_in_expr, flow),
                (_, Flow::ContinueLoop(x)) => continue_!(x, for_in_expr, flow),
            }

            self.stack[st - 1] = iter_val_inc!(iter, self, for_in_expr, module);
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        Ok((Some(Variable::F64(min, sec)), flow))
    }

    pub(crate) fn max_in_expr(
        &mut self,
        for_in_expr: &ast::ForIn,
        module: &Arc<Module>
    ) -> Result<(Option<Variable>, Flow), String> {
        use std::error::Error;

        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();

        let iter = iter!(self, for_in_expr, module);
        let iter_val = iter_val!(iter, self, for_in_expr, module);

        let mut max = ::std::f64::NAN;
        let mut sec = None;
        // Initialize counter.
        self.local_stack.push((for_in_expr.name.clone(), self.stack.len()));
        self.stack.push(iter_val);
        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match self.block(&for_in_expr.block, module)? {
                (Some(x), Flow::Continue) => {
                    match self.resolve(&x) {
                        &Variable::F64(val, ref val_sec) => {
                            if max.is_nan() || max < val {
                                max = val;
                                sec = match *val_sec {
                                    None => {
                                        Some(Box::new(vec![self.stack[st - 1].clone()]))
                                    }
                                    Some(ref arr) => {
                                        let mut arr = arr.clone();
                                        arr.push(self.stack[st - 1].clone());
                                        Some(arr)
                                    }
                                };
                            }
                        },
                        x => return Err(module.error(for_in_expr.block.source_range,
                                &self.expected(x, "number"), self))
                    };
                }
                (x, Flow::Return) => { return Ok((x, Flow::Return)); }
                (None, Flow::Continue) => {
                    return Err(module.error(for_in_expr.block.source_range,
                                "Expected `number or option`", self))
                }
                (_, Flow::Break(x)) => break_!(x, for_in_expr, flow),
                (_, Flow::ContinueLoop(x)) => continue_!(x, for_in_expr, flow),
            }

            self.stack[st - 1] = iter_val_inc!(iter, self, for_in_expr, module);
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        Ok((Some(Variable::F64(max, sec)), flow))
    }

    pub(crate) fn any_in_expr(
        &mut self,
        for_in_expr: &ast::ForIn,
        module: &Arc<Module>
    ) -> Result<(Option<Variable>, Flow), String> {
        use std::error::Error;

        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();

        let iter = iter!(self, for_in_expr, module);
        let iter_val = iter_val!(iter, self, for_in_expr, module);

        let mut any = false;
        let mut sec = None;
        // Initialize counter.
        self.local_stack.push((for_in_expr.name.clone(), self.stack.len()));
        self.stack.push(iter_val);

        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match self.block(&for_in_expr.block, module)? {
                (Some(x), Flow::Continue) => {
                    match self.resolve(&x) {
                        &Variable::Bool(val, ref val_sec) => {
                            if val {
                                any = true;
                                sec = match *val_sec {
                                    None => {
                                        Some(Box::new(vec![self.stack[st - 1].clone()]))
                                    }
                                    Some(ref arr) => {
                                        let mut arr = arr.clone();
                                        arr.push(self.stack[st - 1].clone());
                                        Some(arr)
                                    }
                                };
                                break;
                            }
                        },
                        x => return Err(module.error(for_in_expr.block.source_range,
                                &self.expected(x, "boolean"), self))
                    };
                }
                (x, Flow::Return) => { return Ok((x, Flow::Return)); }
                (None, Flow::Continue) => {
                    return Err(module.error(for_in_expr.block.source_range,
                                "Expected `boolean`", self))
                }
                (_, Flow::Break(x)) => break_!(x, for_in_expr, flow),
                (_, Flow::ContinueLoop(x)) => continue_!(x, for_in_expr, flow),
            }

            self.stack[st - 1] = iter_val_inc!(iter, self, for_in_expr, module);
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        Ok((Some(Variable::Bool(any, sec)), flow))
    }

    pub(crate) fn all_in_expr(
        &mut self,
        for_in_expr: &ast::ForIn,
        module: &Arc<Module>
    ) -> Result<(Option<Variable>, Flow), String> {
        use std::error::Error;

        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();

        let iter = iter!(self, for_in_expr, module);
        let iter_val = iter_val!(iter, self, for_in_expr, module);

        let mut all = true;
        let mut sec = None;
        // Initialize counter.
        self.local_stack.push((for_in_expr.name.clone(), self.stack.len()));
        self.stack.push(iter_val);

        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match self.block(&for_in_expr.block, module)? {
                (Some(x), Flow::Continue) => {
                    match self.resolve(&x) {
                        &Variable::Bool(val, ref val_sec) => {
                            if !val {
                                all = false;
                                sec = match *val_sec {
                                    None => {
                                        Some(Box::new(vec![self.stack[st - 1].clone()]))
                                    }
                                    Some(ref arr) => {
                                        let mut arr = arr.clone();
                                        arr.push(self.stack[st - 1].clone());
                                        Some(arr)
                                    }
                                };
                                break;
                            }
                        },
                        x => return Err(module.error(for_in_expr.block.source_range,
                                &self.expected(x, "boolean"), self))
                    };
                }
                (x, Flow::Return) => { return Ok((x, Flow::Return)); }
                (None, Flow::Continue) => {
                    return Err(module.error(for_in_expr.block.source_range,
                                "Expected `boolean`", self))
                }
                (_, Flow::Break(x)) => break_!(x, for_in_expr, flow),
                (_, Flow::ContinueLoop(x)) => continue_!(x, for_in_expr, flow),
            }

            self.stack[st - 1] = iter_val_inc!(iter, self, for_in_expr, module);
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        Ok((Some(Variable::Bool(all, sec)), flow))
    }

    pub(crate) fn link_for_in_expr(
        &mut self,
        for_in_expr: &ast::ForIn,
        module: &Arc<Module>
    ) -> Result<(Option<Variable>, Flow), String> {
        use Link;

        fn sub_link_for_in_expr(
            res: &mut Link,
            rt: &mut Runtime,
            for_in_expr: &ast::ForIn,
            module: &Arc<Module>
        ) -> Result<(Option<Variable>, Flow), String> {
            use std::error::Error;

            let prev_st = rt.stack.len();
            let prev_lc = rt.local_stack.len();

            let iter = iter!(rt, for_in_expr, module);
            let iter_val = iter_val!(iter, rt, for_in_expr, module);

            // Initialize counter.
            rt.local_stack.push((for_in_expr.name.clone(), rt.stack.len()));
            rt.stack.push(iter_val);

            let st = rt.stack.len();
            let lc = rt.local_stack.len();
            let mut flow = Flow::Continue;

            'outer: loop {
                match for_in_expr.block.expressions[0] {
                    ast::Expression::Link(ref link) => {
                        // Evaluate link items directly.
                        'inner: for item in &link.items {
                            match rt.expression(item, Side::Right, module)? {
                                (Some(ref x), Flow::Continue) => {
                                    match res.push(rt.resolve(x)) {
                                        Err(err) => {
                                            return Err(module.error(for_in_expr.source_range,
                                                &format!("{}\n{}", rt.stack_trace(),
                                                err), rt))
                                        }
                                        Ok(()) => {}
                                    }
                                }
                                (x, Flow::Return) => { return Ok((x, Flow::Return)); }
                                (None, Flow::Continue) => {}
                                (_, Flow::Break(x)) => {
                                    if let Some(label) = x {
                                        let same =
                                        if let Some(ref for_label) = for_in_expr.label {
                                            &label == for_label
                                        } else { false };
                                        if !same {
                                            flow = Flow::Break(Some(label))
                                        }
                                    }
                                    break 'outer;
                                }
                                (_, Flow::ContinueLoop(x)) => {
                                    match x {
                                        Some(label) => {
                                            let same =
                                            if let Some(ref for_label) = for_in_expr.label {
                                                &label == for_label
                                            } else { false };
                                            if !same {
                                                flow = Flow::ContinueLoop(Some(label));
                                                break 'outer;
                                            } else {
                                                break 'inner;
                                            }
                                        }
                                        None => {
                                            break 'inner;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ast::Expression::LinkIn(ref for_in) => {
                        // Pass on control to next link loop.
                        match sub_link_for_in_expr(res, rt, for_in, module) {
                            Ok((None, Flow::Continue)) => {}
                            Ok((_, Flow::Break(x))) => {
                                if let Some(label) = x {
                                    let same =
                                    if let Some(ref for_label) = for_in_expr.label {
                                        &label == for_label
                                    } else { false };
                                    if !same {
                                        flow = Flow::Break(Some(label))
                                    }
                                }
                                break 'outer;
                            }
                            Ok((_, Flow::ContinueLoop(x))) => {
                                if let Some(label) = x {
                                    let same =
                                    if let Some(ref for_label) = for_in_expr.label {
                                        &label == for_label
                                    } else { false };
                                    if !same {
                                        flow = Flow::ContinueLoop(Some(label));
                                        break 'outer;
                                    }
                                }
                            }
                            x => return x
                        }
                    }
                    _ => {
                        panic!("Link body is not link");
                    }
                }

                rt.stack[st - 1] = iter_val_inc!(iter, rt, for_in_expr, module);
                rt.stack.truncate(st);
                rt.local_stack.truncate(lc);
            };
            rt.stack.truncate(prev_st);
            rt.local_stack.truncate(prev_lc);
            Ok((None, flow))
        }

        let mut res: Link = Link::new();
        match sub_link_for_in_expr(&mut res, self, for_in_expr, module) {
            Ok((None, Flow::Continue)) =>
                Ok((Some(Variable::Link(Box::new(res))), Flow::Continue)),
            x => x
        }
    }

    pub(crate) fn sift_in_expr(
        &mut self,
        for_in_expr: &ast::ForIn,
        module: &Arc<Module>
    ) -> Result<(Option<Variable>, Flow), String> {
        use std::error::Error;

        let prev_st = self.stack.len();
        let prev_lc = self.local_stack.len();
        let mut res: Vec<Variable> = vec![];

        let iter = iter!(self, for_in_expr, module);
        let iter_val = iter_val!(iter, self, for_in_expr, module);

        // Initialize counter.
        self.local_stack.push((for_in_expr.name.clone(), self.stack.len()));
        self.stack.push(iter_val);

        let st = self.stack.len();
        let lc = self.local_stack.len();
        let mut flow = Flow::Continue;
        loop {
            match self.block(&for_in_expr.block, module)? {
                (Some(x), Flow::Continue) => res.push(x),
                (x, Flow::Return) => { return Ok((x, Flow::Return)); }
                (None, Flow::Continue) => {
                    return Err(module.error(for_in_expr.block.source_range,
                                "Expected variable", self))
                }
                (_, Flow::Break(x)) => break_!(x, for_in_expr, flow),
                (_, Flow::ContinueLoop(x)) => continue_!(x, for_in_expr, flow),
            }

            self.stack[st - 1] = iter_val_inc!(iter, self, for_in_expr, module);
            self.stack.truncate(st);
            self.local_stack.truncate(lc);
        };
        self.stack.truncate(prev_st);
        self.local_stack.truncate(prev_lc);
        Ok((Some(Variable::Array(Arc::new(res))), flow))
    }
}
