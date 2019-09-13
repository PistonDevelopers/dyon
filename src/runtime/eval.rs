use super::*;

impl RuntimeEval<ast::Expression, Variable> for Runtime {
    fn expression(
        &mut self,
        expr: &ast::Expression,
        side: Side,
    ) -> Result<(Option<Variable>, Flow), String> {
        use ast::Expression::*;

        match *expr {
            Link(ref link) => self.link(link),
            Object(ref obj) => self.object(obj),
            Array(ref arr) => self.array(arr),
            ArrayFill(ref array_fill) => self.array_fill(array_fill),
            Block(ref block) => self.block(block),
            Return(ref ret) => {
                let x = match self.expression(ret, Side::Right)? {
                    (Some(x), Flow::Continue) => x,
                    (x, Flow::Return) => { return Ok((x, Flow::Return)); }
                    _ => return Err(self.module.error(expr.source_range(),
                                    &format!("{}\nExpected something",
                                        self.stack_trace()), self))
                };
                Ok((Some(x), Flow::Return))
            }
            ReturnVoid(_) => Ok((None, Flow::Return)),
            Break(ref b) => Ok((None, Flow::Break(b.label.clone()))),
            Continue(ref b) => Ok((None, Flow::ContinueLoop(b.label.clone()))),
            Go(ref go) => self.go(go),
            Call(ref call) => {
                let loader = false;
                self.call_internal(call, loader)
            }
            Item(ref item) => self.item(item, side),
            Norm(ref norm) => self.norm(norm, side),
            UnOp(ref unop) => self.unop(unop, side),
            BinOp(ref binop) => self.binop(binop, side),
            Assign(ref assign) => self.assign(assign.op, &assign.left, &assign.right),
            Vec4(ref vec4) => self.vec4(vec4, side),
            Mat4(ref mat4) => self.mat4(mat4, side),
            For(ref for_expr) => self.for_expr(for_expr),
            ForN(ref for_n_expr) => self.for_n_expr(for_n_expr),
            ForIn(ref for_in_expr) => self.for_in_expr(for_in_expr),
            Sum(ref for_n_expr) => self.sum_n_expr(for_n_expr),
            SumIn(ref sum_in_expr) => self.sum_in_expr(sum_in_expr),
            SumVec4(ref for_n_expr) => self.sum_vec4_n_expr(for_n_expr),
            Prod(ref for_n_expr) => self.prod_n_expr(for_n_expr),
            ProdIn(ref for_in_expr) => self.prod_in_expr(for_in_expr),
            ProdVec4(ref for_n_expr) => self.prod_vec4_n_expr(for_n_expr),
            Min(ref for_n_expr) => self.min_n_expr(for_n_expr),
            MinIn(ref for_in_expr) => self.min_in_expr(for_in_expr),
            Max(ref for_n_expr) => self.max_n_expr(for_n_expr),
            MaxIn(ref for_in_expr) => self.max_in_expr(for_in_expr),
            Sift(ref for_n_expr) => self.sift_n_expr(for_n_expr),
            SiftIn(ref for_in_expr) => self.sift_in_expr(for_in_expr),
            Any(ref for_n_expr) => self.any_n_expr(for_n_expr),
            AnyIn(ref for_in_expr) => self.any_in_expr(for_in_expr),
            All(ref for_n_expr) => self.all_n_expr(for_n_expr),
            AllIn(ref for_in_expr) => self.all_in_expr(for_in_expr),
            LinkFor(ref for_n_expr) => self.link_for_n_expr(for_n_expr),
            LinkIn(ref for_in_expr) => self.link_for_in_expr(for_in_expr),
            If(ref if_expr) => self.if_expr(if_expr),
            Compare(ref compare) => self.compare(compare),
            Variable(ref range_var) => Ok((Some(range_var.1.clone()), Flow::Continue)),
            Try(ref expr) => self.try(expr, side),
            Swizzle(ref sw) => {
                let flow = self.swizzle(sw)?;
                Ok((None, flow))
            }
            Closure(ref closure) => self.closure(closure),
            CallClosure(ref call) => self.call_closure(call),
            Grab(ref expr) => Err(self.module.error(expr.source_range,
                    &format!("{}\n`grab` expressions must be inside a closure",
                        self.stack_trace()), self)),
            TryExpr(ref try_expr) => self.try_expr(try_expr),
            In(ref in_expr) => self.in_expr(in_expr),
        }
    }
}
