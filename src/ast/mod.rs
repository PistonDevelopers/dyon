use std::sync::Arc;
use std::cell::Cell;
use range::Range;
use piston_meta::bootstrap::Convert;
use piston_meta::MetaData;

use FnIndex;
use Module;
use Type;
use Variable;

mod infer_len;

pub fn convert(
    file: Arc<String>,
    data: &[Range<MetaData>],
    ignored: &mut Vec<Range>,
    module: &mut Module
) -> Result<(), ()> {
    let mut convert = Convert::new(data);
    loop {
        if let Ok((range, function)) =
        Function::from_meta_data(file.clone(), convert, ignored) {
            convert.update(range);
            module.register(function);
        } else if convert.remaining_data_len() > 0 {
            return Err(());
        } else {
            break;
        }
    }
    for f in &module.functions {
        f.resolve_locals(module);
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Arc<String>,
    pub file: Arc<String>,
    pub args: Vec<Arg>,
    pub block: Block,
    pub ret: Type,
    pub resolved: Cell<bool>,
    pub source_range: Range,
}

impl Function {
    pub fn from_meta_data(
        file: Arc<String>,
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Function), ()> {
        let start = convert.clone();
        let node = "fn";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
        let mut args: Vec<Arg> = vec![];
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
            } else if let Ok((range, val)) = convert.meta_bool("returns") {
                convert.update(range);
                ret = Some(if val { Type::Any } else { Type::Void })
            } else if let Ok((range, val)) = Type::from_meta_data(
                    "ret_type", convert, ignored) {
                convert.update(range);
                ret = Some(val);
            } else if let Ok((range, val)) = Block::from_meta_data(
                "block", convert, ignored) {
                convert.update(range);
                block = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "expr", convert, ignored) {
                convert.update(range);
                expr = Some(val);
                ret = Some(Type::Any);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let mut name = try!(name.ok_or(()));
        let block = match expr {
            None => try!(block.ok_or(())),
            Some(expr) => {
                let source_range = expr.source_range();
                let item = Expression::Item(Item {
                        name: Arc::new("return".into()),
                        stack_id: Cell::new(None),
                        static_stack_id: Cell::new(None),
                        try: false,
                        ids: vec![],
                        try_ids: vec![],
                        source_range: source_range,
                    });
                Block {
                    expressions: vec![Expression::Return(Box::new(item), Box::new(expr))],
                    source_range: source_range
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
        let ret = try!(ret.ok_or(()));
        Ok((convert.subtract(start), Function {
            resolved: Cell::new(false),
            name: name,
            file: file,
            args: args,
            block: block,
            ret: ret,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn returns(&self) -> bool { self.ret != Type::Void }

    pub fn resolve_locals(&self, module: &Module) {
        if self.resolved.get() { return; }
        let mut stack: Vec<Option<Arc<String>>> = vec![];
        if self.returns() {
            stack.push(Some(Arc::new("return".into())));
        }
        for arg in &self.args {
            stack.push(Some(arg.name.clone()));
        }
        self.block.resolve_locals(&mut stack, module);
        self.resolved.set(true);
    }
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: Arc<String>,
    pub lifetime: Option<Arc<String>>,
    pub ty: Type,
    pub source_range: Range,
    pub mutable: bool,
}

impl Arg {
    pub fn from_meta_data(mut convert: Convert, ignored: &mut Vec<Range>)
    -> Result<(Range, Arg), ()> {
        let start = convert.clone();
        let node = "arg";
        let start_range = try!(convert.start_node(node));
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

        let name = try!(name.ok_or(()));
        let ty = match ty {
            None => Type::Any,
            Some(ty) => ty
        };
        Ok((convert.subtract(start), Arg {
            name: name,
            lifetime: lifetime,
            ty: ty,
            source_range: convert.source(start).unwrap(),
            mutable: mutable,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub expressions: Vec<Expression>,
    pub source_range: Range,
}

impl Block {
    pub fn from_meta_data(node: &str, mut convert: Convert, ignored: &mut Vec<Range>)
    -> Result<(Range, Block), ()> {
        let start = convert.clone();
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut expressions = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "expr", convert, ignored) {
                convert.update(range);
                expressions.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), Block {
            expressions: expressions,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        for expr in &self.expressions {
            expr.resolve_locals(stack, module);
        }
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Object(Box<Object>),
    Array(Box<Array>),
    ArrayFill(Box<ArrayFill>),
    Return(Box<Expression>, Box<Expression>),
    ReturnVoid(Range),
    Break(Break),
    Continue(Continue),
    Block(Block),
    Go(Box<Go>),
    Call(Call),
    Item(Item),
    BinOp(Box<BinOpExpression>),
    Assign(Box<Assign>),
    Text(Text),
    Number(Number),
    Vec4(Vec4),
    Bool(Bool),
    For(Box<For>),
    ForN(Box<ForN>),
    Sum(Box<ForN>),
    SumVec4(Box<ForN>),
    Min(Box<ForN>),
    Max(Box<ForN>),
    Sift(Box<ForN>),
    Any(Box<ForN>),
    All(Box<ForN>),
    If(Box<If>),
    Compare(Box<Compare>),
    UnOp(Box<UnOpExpression>),
    Variable(Range, Variable),
    Try(Box<Expression>),
}

// Required because the `Sync` impl of `Variable` is unsafe.
unsafe impl Sync for Expression {}

impl Expression {
    pub fn from_meta_data(
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Expression), ()> {
        let start = convert.clone();
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut result: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, _)) = convert.meta_bool("mut") {
                // Ignore `mut` since it is handled by lifetime checker.
                convert.update(range);
            } else if let Ok((range, val)) = Object::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Object(Box::new(val)));
            } else if let Ok((range, val)) = Array::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Array(Box::new(val)));
            } else if let Ok((range, val)) = ArrayFill::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::ArrayFill(Box::new(val)));
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "return", convert, ignored) {
                convert.update(range);
                let item = Expression::Item(Item {
                        name: Arc::new("return".into()),
                        stack_id: Cell::new(None),
                        static_stack_id: Cell::new(None),
                        try: false,
                        ids: vec![],
                        try_ids: vec![],
                        source_range: val.source_range(),
                    });
                result = Some(Expression::Return(Box::new(item), Box::new(val)));
            } else if let Ok((range, _)) = convert.meta_bool("return_void") {
                convert.update(range);
                result = Some(Expression::ReturnVoid(
                    convert.source(start).unwrap()));
            } else if let Ok((range, val)) = Break::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Break(val));
            } else if let Ok((range, val)) = Continue::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Continue(val));
            } else if let Ok((range, val)) = Block::from_meta_data(
                "block", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Block(val));
            } else if let Ok((range, val)) = Add::from_meta_data(
                convert, ignored) {
                convert.update(range);
                result = Some(val.to_expression());
            } else if let Ok((range, val)) = Item::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Item(val));
            } else if let Ok((range, val)) = convert.meta_string("text") {
                convert.update(range);
                result = Some(Expression::Text(Text {
                    text: val,
                    source_range: convert.source(start).unwrap(),
                }));
            } else if let Ok((range, val)) = convert.meta_f64("num") {
                convert.update(range);
                result = Some(Expression::Number(Number {
                    num: val,
                    source_range: convert.source(start).unwrap(),
                }));
            } else if let Ok((range, val)) = Vec4::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Vec4(val));
            } else if let Ok((range, val)) = convert.meta_bool("bool") {
                convert.update(range);
                result = Some(Expression::Bool(Bool {
                    val: val,
                    source_range: convert.source(start).unwrap(),
                }));
            } else if let Ok((range, val)) = convert.meta_string("color") {
                use read_color;

                convert.update(range);
                if let Some((rgb, a)) = read_color::rgb_maybe_a(&mut val.chars()) {
                    let v = [rgb[0] as f32 / 255.0, rgb[1] as f32 / 255.0, rgb[2] as f32 / 255.0,
                             a.unwrap_or(255) as f32 / 255.0];
                    result = Some(Expression::Variable(range, Variable::Vec4(v)));
                } else {
                    return Err(());
                }
            } else if let Ok((range, val)) = Go::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Go(Box::new(val)));
            } else if let Ok((range, val)) = Call::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Call(val));
            } else if let Ok((range, val)) = Call::named_from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Call(val));
            } else if let Ok((range, val)) = Assign::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Assign(Box::new(val)));
            } else if let Ok((range, val)) = For::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::For(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    "for_n", convert, ignored) {
                convert.update(range);
                result = Some(Expression::ForN(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    "sum", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Sum(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    "sum_vec4", convert, ignored) {
                convert.update(range);
                result = Some(Expression::SumVec4(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    "min", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Min(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    "max", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Max(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    "sift", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Sift(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    "any", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Any(Box::new(val)));
            } else if let Ok((range, val)) = ForN::from_meta_data(
                    "all", convert, ignored) {
                convert.update(range);
                result = Some(Expression::All(Box::new(val)));
            } else if let Ok((range, val)) = Loop::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(val.to_expression());
            } else if let Ok((range, val)) = If::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::If(Box::new(val)));
            } else if let Ok((range, val)) = Compare::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Compare(Box::new(val)));
            } else if let Ok((range, val)) = UnOpExpression::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::UnOp(Box::new(val)));
            } else if let Ok((range, _)) = convert.meta_bool("try") {
                convert.update(range);
                result = Some(Expression::Try(Box::new(result.unwrap())));
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let result = try!(result.ok_or(()));
        Ok((convert.subtract(start), result))
    }

    pub fn source_range(&self) -> Range {
        use self::Expression::*;

        match *self {
            Object(ref obj) => obj.source_range,
            Array(ref arr) => arr.source_range,
            ArrayFill(ref arr_fill) => arr_fill.source_range,
            Return(_, ref expr) => expr.source_range(),
            ReturnVoid(range) => range,
            Break(ref br) => br.source_range,
            Continue(ref c) => c.source_range,
            Block(ref bl) => bl.source_range,
            Go(ref go) => go.source_range,
            Call(ref call) => call.source_range,
            Item(ref it) => it.source_range,
            BinOp(ref binop) => binop.source_range,
            Assign(ref assign) => assign.source_range,
            Text(ref text) => text.source_range,
            Number(ref num) => num.source_range,
            Vec4(ref vec4) => vec4.source_range,
            Bool(ref b) => b.source_range,
            For(ref for_expr) => for_expr.source_range,
            ForN(ref for_n_expr) => for_n_expr.source_range,
            Sum(ref for_n_expr) => for_n_expr.source_range,
            SumVec4(ref for_n_expr) => for_n_expr.source_range,
            Min(ref for_n_expr) => for_n_expr.source_range,
            Max(ref for_n_expr) => for_n_expr.source_range,
            Sift(ref for_n_expr) => for_n_expr.source_range,
            Any(ref for_n_expr) => for_n_expr.source_range,
            All(ref for_n_expr) => for_n_expr.source_range,
            If(ref if_expr) => if_expr.source_range,
            Compare(ref comp) => comp.source_range,
            UnOp(ref unop) => unop.source_range,
            Variable(range, _) => range,
            Try(ref expr) => expr.source_range(),
        }
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        use self::Expression::*;

        match *self {
            Object(ref obj) => obj.resolve_locals(stack, module),
            Array(ref arr) => arr.resolve_locals(stack, module),
            ArrayFill(ref arr_fill) => arr_fill.resolve_locals(stack, module),
            Return(ref item, ref expr) => {
                let st = stack.len();
                expr.resolve_locals(stack, module);
                stack.truncate(st);
                stack.push(None);
                item.resolve_locals(stack, module);
                stack.truncate(st);
            }
            ReturnVoid(_) => {}
            Break(_) => {}
            Continue(_) => {}
            Block(ref bl) => bl.resolve_locals(stack, module),
            Go(ref go) => go.resolve_locals(stack, module),
            Call(ref call) => call.resolve_locals(stack, module),
            Item(ref it) => it.resolve_locals(stack, module),
            BinOp(ref binop) => binop.resolve_locals(stack, module),
            Assign(ref assign) => assign.resolve_locals(stack, module),
            Text(_) => {}
            Number(_) => {}
            Vec4(ref vec4) => vec4.resolve_locals(stack, module),
            Bool(_) => {}
            For(ref for_expr) => for_expr.resolve_locals(stack, module),
            ForN(ref for_n_expr) => for_n_expr.resolve_locals(stack, module),
            Sum(ref for_n_expr) => for_n_expr.resolve_locals(stack, module),
            SumVec4(ref for_n_expr) => for_n_expr.resolve_locals(stack, module),
            Min(ref for_n_expr) => for_n_expr.resolve_locals(stack, module),
            Max(ref for_n_expr) => for_n_expr.resolve_locals(stack, module),
            Sift(ref for_n_expr) => for_n_expr.resolve_locals(stack, module),
            Any(ref for_n_expr) => for_n_expr.resolve_locals(stack, module),
            All(ref for_n_expr) => for_n_expr.resolve_locals(stack, module),
            If(ref if_expr) => if_expr.resolve_locals(stack, module),
            Compare(ref comp) => comp.resolve_locals(stack, module),
            UnOp(ref unop) => unop.resolve_locals(stack, module),
            Variable(_, _) => {}
            Try(ref expr) => expr.resolve_locals(stack, module),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    pub key_values: Vec<(Arc<String>, Expression)>,
    pub source_range: Range,
}

impl Object {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Object), ()> {
        let start = convert.clone();
        let node = "object";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut key_values = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Object::key_value_from_meta_data(
                convert, ignored) {
                convert.update(range);
                key_values.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), Object {
            key_values: key_values,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn key_value_from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, (Arc<String>, Expression)), ()> {
        let start = convert.clone();
        let node = "key_value";
        let start_range = try!(convert.start_node(node));
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
                "val", convert, ignored) {
                convert.update(range);
                value = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let key = try!(key.ok_or(()));
        let value = try!(value.ok_or(()));
        Ok((convert.subtract(start), (key, value)))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        for &(_, ref expr) in &self.key_values {
            expr.resolve_locals(stack, module);
            stack.truncate(st);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    pub items: Vec<Expression>,
    pub source_range: Range,
}

impl Array {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Array), ()> {
        let start = convert.clone();
        let node = "array";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut items = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "array_item", convert, ignored) {
                convert.update(range);
                items.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        Ok((convert.subtract(start), Array {
            items: items,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        for item in &self.items {
            item.resolve_locals(stack, module);
            stack.truncate(st);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArrayFill {
    pub fill: Expression,
    pub n: Expression,
    pub source_range: Range,
}

impl ArrayFill {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, ArrayFill), ()> {
        let start = convert.clone();
        let node = "array_fill";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut fill: Option<Expression> = None;
        let mut n: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "fill", convert, ignored) {
                convert.update(range);
                fill = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "n", convert, ignored) {
                convert.update(range);
                n = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let fill = try!(fill.ok_or(()));
        let n = try!(n.ok_or(()));
        Ok((convert.subtract(start), ArrayFill {
            fill: fill,
            n: n,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        self.fill.resolve_locals(stack, module);
        stack.truncate(st);
        self.n.resolve_locals(stack, module);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Add {
    pub items: Vec<Mul>,
    pub ops: Vec<BinOp>,
    pub source_range: Range,
}

impl Add {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Add), ()> {
        let start = convert.clone();
        let node = "add";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut items = vec![];
        let mut ops = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Mul::from_meta_data(
                convert, ignored) {
                convert.update(range);
                items.push(val);
            } else if let Ok((range, _)) = convert.meta_bool("+") {
                convert.update(range);
                ops.push(BinOp::Add);
            } else if let Ok((range, _)) = convert.meta_bool("-") {
                convert.update(range);
                ops.push(BinOp::Sub);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        if items.len() == 0 {
            return Err(())
        }
        Ok((convert.subtract(start), Add {
            items: items,
            ops: ops,
            source_range: convert.source(start).unwrap()
        }))
    }

    pub fn to_expression(mut self) -> Expression {
        if self.items.len() == 1 {
            self.items[0].clone().to_expression()
        } else {
            let op = self.ops.pop().unwrap();
            let last = self.items.pop().unwrap();
            let source_range = self.source_range;
            Expression::BinOp(Box::new(BinOpExpression {
                op: op,
                left: self.to_expression(),
                right: last.to_expression(),
                source_range: source_range
            }))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mul {
    pub items: Vec<MulVar>,
    pub ops: Vec<BinOp>,
    pub source_range: Range,
}

impl Mul {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Mul), ()> {
        let start = convert.clone();
        let node = "mul";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut items = vec![];
        let mut ops = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = MulVar::from_meta_data(
                convert, ignored) {
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
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        if items.len() == 0 {
            return Err(())
        }
        Ok((convert.subtract(start), Mul {
            items: items,
            ops: ops,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn to_expression(mut self) -> Expression {
        if self.items.len() == 1 {
            self.items[0].clone().to_expression()
        } else {
            let op = self.ops.pop().expect("Expected a binary operation");
            let last = self.items.pop().expect("Expected argument");
            let source_range = self.source_range;
            Expression::BinOp(Box::new(BinOpExpression {
                op: op,
                left: self.to_expression(),
                right: last.to_expression(),
                source_range: source_range,
            }))
        }
    }
}

#[derive(Debug, Clone)]
pub enum MulVar {
    Pow(Pow),
    Val(Expression),
}

impl MulVar {
    pub fn from_meta_data(
        convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, MulVar), ()> {
        if let Ok((range, val)) = Expression::from_meta_data(
            "val", convert, ignored) {
            Ok((range, MulVar::Val(val)))
        } else if let Ok((range, val)) = Pow::from_meta_data(convert, ignored) {
            Ok((range, MulVar::Pow(val)))
        } else {
            Err(())
        }
    }

    pub fn to_expression(self) -> Expression {
        match self {
            MulVar::Pow(a) => Expression::BinOp(Box::new(BinOpExpression {
                op: BinOp::Pow,
                left: a.base,
                right: a.exp,
                source_range: a.source_range,
            })),
            MulVar::Val(a) => a
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pow {
    pub base: Expression,
    pub exp: Expression,
    pub source_range: Range,
}

impl Pow {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Pow), ()> {
        let start = convert.clone();
        let node = "pow";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut base: Option<Expression> = None;
        let mut exp: Option<Expression> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "base", convert, ignored) {
                convert.update(range);
                base = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "exp", convert, ignored) {
                convert.update(range);
                exp = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let base = try!(base.ok_or(()));
        let exp = try!(exp.ok_or(()));
        Ok((convert.subtract(start), Pow {
            base: base,
            exp: exp,
            source_range: convert.source(start).unwrap()
        }))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Dot,
    Cross,
    Div,
    Rem,
    Pow
}

impl BinOp {
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
        }
    }

    pub fn symbol_bool(self) -> &'static str {
        match self {
            BinOp::Add => "||",
            BinOp::Mul => "&&",
            _ => self.symbol()
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum UnOp {
    Not,
    Neg,
    Norm,
}

#[derive(Debug, Clone)]
pub enum Id {
    String(Range, Arc<String>),
    F64(Range, f64),
    Expression(Expression),
}

impl Id {
    pub fn source_range(&self) -> Range {
        match *self {
            Id::String(range, _) => range,
            Id::F64(range, _) => range,
            Id::Expression(ref expr) => expr.source_range(),
        }
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) -> bool {
        match *self {
            Id::String(_, _) => false,
            Id::F64(_, _) => false,
            Id::Expression(ref expr) => {
                let st = stack.len();
                expr.resolve_locals(stack, module);
                stack.truncate(st);
                true
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: Arc<String>,
    pub stack_id: Cell<Option<usize>>,
    pub static_stack_id: Cell<Option<usize>>,
    pub try: bool,
    pub ids: Vec<Id>,
    // Stores indices of ids that should propagate errors.
    pub try_ids: Vec<usize>,
    pub source_range: Range,
}

impl Item {
    pub fn from_variable(name: Arc<String>, source_range: Range) -> Item {
        Item {
            name: name,
            stack_id: Cell::new(None),
            static_stack_id: Cell::new(None),
            try: false,
            ids: vec![],
            try_ids: vec![],
            source_range: source_range
        }
    }

    /// Truncates item extra to a given length.
    pub fn trunc(&self, n: usize) -> Item {
        Item {
            name: self.name.clone(),
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

    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Item), ()> {
        let start = convert.clone();
        let node = "item";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
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
                "id", convert, ignored) {
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

        let name = try!(name.ok_or(()));
        Ok((convert.subtract(start), Item {
            name: name,
            stack_id: Cell::new(None),
            static_stack_id: Cell::new(None),
            try: try,
            ids: ids,
            try_ids: try_ids,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        // println!("TEST item resolve {} {:?}", self.name, stack);
        let st = stack.len();
        for (i, n) in stack.iter().rev().enumerate() {
            if let &Some(ref n) = n {
                if &**n == &**self.name {
                    // println!("TEST set {} {}", self.name, i + 1);
                    self.static_stack_id.set(Some(i + 1));
                    break;
                }
            }
        }
        for id in &self.ids {
            if id.resolve_locals(stack, module) {
                stack.push(None);
            }
        }
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Go {
    pub call: Call,
    pub source_range: Range,
}

impl Go {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>
    ) -> Result<(Range, Go), ()> {
        let start = convert.clone();
        let node = "go";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut call: Option<Call> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Call::from_meta_data(convert, ignored) {
                convert.update(range);
                call = Some(val);
            } else if let Ok((range, val)) = Call::named_from_meta_data(convert, ignored) {
                convert.update(range);
                call = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let call = try!(call.ok_or(()));
        Ok((convert.subtract(start), Go {
            call: call,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        for arg in &self.call.args {
            let st = stack.len();
            arg.resolve_locals(stack, module);
            stack.truncate(st);
        }
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub name: Arc<String>,
    pub args: Vec<Expression>,
    pub f_index: Cell<FnIndex>,
    pub source_range: Range,
}

impl Call {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Call), ()> {
        let start = convert.clone();
        let node = "call";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
        let mut args = vec![];
        let mut mutable: Vec<bool> = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "call_arg", convert, ignored) {
                let mut peek = convert.clone();
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

        let mut name = try!(name.ok_or(()));

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
            name: name,
            args: args,
            f_index: Cell::new(FnIndex::None),
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn named_from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Call), ()> {
        let start = convert.clone();
        let node = "named_call";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut name = String::new();
        let mut args = vec![];
        let mut mutable: Vec<bool> = vec![];
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("word") {
                convert.update(range);
                if name.len() != 0 { name.push('_'); }
                name.push_str(&val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "call_arg", convert, ignored) {
                let mut peek = convert.clone();
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
            name: Arc::new(name),
            args: args,
            f_index: Cell::new(FnIndex::None),
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        let f_index = module.find_function(&self.name);
        self.f_index.set(f_index);
        match f_index {
            FnIndex::Loaded(f_index) => {
                if module.functions[f_index].returns() {
                    stack.push(Some(Arc::new("return".into())));
                }
            }
            FnIndex::External(f_index) => {
                let f = &module.ext_prelude[f_index];
                if f.p.returns() {
                    stack.push(Some(Arc::new("return".into())));
                }
            }
            FnIndex::None => {}
        }
        for arg in &self.args {
            let arg_st = stack.len();
            arg.resolve_locals(stack, module);
            stack.truncate(arg_st);
            stack.push(None);
        }
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct BinOpExpression {
    pub op: BinOp,
    pub left: Expression,
    pub right: Expression,
    pub source_range: Range,
}

impl BinOpExpression {
    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        self.left.resolve_locals(stack, module);
        stack.truncate(st);
        stack.push(None);
        self.right.resolve_locals(stack, module);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct UnOpExpression {
    pub op: UnOp,
    pub expr: Expression,
    pub source_range: Range,
}

impl UnOpExpression {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, UnOpExpression), ()> {
        let start = convert.clone();
        let node = "unop";
        let start_range = try!(convert.start_node(node));
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
            } else if let Ok((range, _)) = convert.meta_bool("norm") {
                convert.update(range);
                unop = Some(UnOp::Norm);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "expr", convert, ignored) {
                convert.update(range);
                expr = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let unop = try!(unop.ok_or(()));
        let expr = try!(expr.ok_or(()));
        Ok((convert.subtract(start), UnOpExpression {
            op: unop,
            expr: expr,
            source_range: convert.source(start).unwrap()
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        self.expr.resolve_locals(stack, module);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub op: AssignOp,
    pub left: Expression,
    pub right: Expression,
    pub source_range: Range,
}

impl Assign {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Assign), ()> {
        let start = convert.clone();
        let node = "assign";
        let start_range = try!(convert.start_node(node));
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
                "left", convert, ignored) {
                convert.update(range);
                left = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "right", convert, ignored) {
                convert.update(range);
                right = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let op = try!(op.ok_or(()));
        let left = try!(left.ok_or(()));
        let right = try!(right.ok_or(()));
        Ok((convert.subtract(start), Assign {
            op: op,
            left: left,
            right: right,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        // Declared locals in right expressions are popped from the stack.
        let st = stack.len();
        self.right.resolve_locals(stack, module);
        stack.truncate(st);

        // Declare new local when there is an item with no extra.
        if let Expression::Item(ref item) = self.left {
            if item.ids.len() == 0 && self.op == AssignOp::Assign {
                stack.push(Some(item.name.clone()));
                return;
            }
        }
        // Or else, just resolve normally.
        if self.op != AssignOp::Assign {
            // Item is resolved before popping right value.
            stack.push(None);
        }
        self.left.resolve_locals(stack, module);
        stack.truncate(st);
    }
}

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

#[derive(Debug, Clone)]
pub struct Number {
    pub num: f64,
    pub source_range: Range,
}

#[derive(Debug, Clone)]
pub struct Vec4 {
    pub args: Vec<Expression>,
    pub source_range: Range,
}

impl Vec4 {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Vec4), ()> {
        let start = convert.clone();
        let node = "vec4";
        let start_range = try!(convert.start_node(node));
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
                "x", convert, ignored) {
                convert.update(range);
                x = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "y", convert, ignored) {
                convert.update(range);
                y = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "z", convert, ignored) {
                convert.update(range);
                z = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "w", convert, ignored) {
                convert.update(range);
                w = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let x = try!(x.ok_or(()));
        let y = try!(y.ok_or(()));
        let z = z.unwrap_or(Expression::Number(
            Number { num: 0.0, source_range: Range::empty(0) }
        ));
        let w = w.unwrap_or(Expression::Number(
            Number { num: 0.0, source_range: Range::empty(0) }
        ));
        Ok((convert.subtract(start), Vec4 {
            args: vec![x, y, z, w],
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        for arg in &self.args {
            let arg_st = stack.len();
            arg.resolve_locals(stack, module);
            stack.truncate(arg_st);
            stack.push(None);
        }
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    pub text: Arc<String>,
    pub source_range: Range,
}

#[derive(Debug, Clone)]
pub struct Bool {
    pub val: bool,
    pub source_range: Range,
}

#[derive(Debug, Clone)]
pub struct For {
    pub init: Expression,
    pub cond: Expression,
    pub step: Expression,
    pub block: Block,
    pub label: Option<Arc<String>>,
    pub source_range: Range,
}

impl For {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, For), ()> {
        let start = convert.clone();
        let node = "for";
        let start_range = try!(convert.start_node(node));
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
                "init", convert, ignored) {
                convert.update(range);
                init = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "cond", convert, ignored) {
                convert.update(range);
                cond = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "step", convert, ignored) {
                convert.update(range);
                step = Some(val);
            } else if let Ok((range, val)) = Block::from_meta_data(
                "block", convert, ignored) {
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

        let init = try!(init.ok_or(()));
        let cond = try!(cond.ok_or(()));
        let step = try!(step.ok_or(()));
        let block = try!(block.ok_or(()));
        Ok((convert.subtract(start), For {
            init: init,
            cond: cond,
            step: step,
            block: block,
            label: label,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        self.init.resolve_locals(stack, module);
        let after_init = stack.len();
        self.cond.resolve_locals(stack, module);
        stack.truncate(after_init);
        self.step.resolve_locals(stack, module);
        stack.truncate(after_init);
        self.block.resolve_locals(stack, module);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct ForN {
    pub name: Arc<String>,
    pub start: Option<Expression>,
    pub end: Expression,
    pub block: Block,
    pub label: Option<Arc<String>>,
    pub source_range: Range,
}

impl ForN {
    pub fn from_meta_data(
        node: &str,
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, ForN), ()> {
        let start = convert.clone();
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut indices: Vec<(Arc<String>, Option<Expression>, Option<Expression>)> = vec![];
        let mut block: Option<Block> = None;
        let mut label: Option<Arc<String>> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Block::from_meta_data(
                "block", convert, ignored) {
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
                    "start", convert, ignored) {
                    convert.update(range);
                    start_expr = Some(val);
                }
                if let Ok((range, val)) = Expression::from_meta_data(
                    "end", convert, ignored) {
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
        if indices.len() == 0 { return Err(()); }

        let name: Arc<String> = indices[0].0.clone();
        let start_expr = indices[0].1.clone();
        let mut end_expr = indices[0].2.clone();

        if indices.len() > 1 {
            let (_, new_for_n) = try!(ForN::create(
                node,
                range,
                source_range,
                None,
                &indices[1..],
                block
            ));
            block = Some(Block {
                source_range: source_range,
                expressions: vec![match node {
                    "for_n" => Expression::ForN(Box::new(new_for_n)),
                    "sum" => Expression::Sum(Box::new(new_for_n)),
                    "any" => Expression::Any(Box::new(new_for_n)),
                    "all" => Expression::All(Box::new(new_for_n)),
                    "min" => Expression::Min(Box::new(new_for_n)),
                    "max" => Expression::Max(Box::new(new_for_n)),
                    "sift" => Expression::Sift(Box::new(new_for_n)),
                    _ => return Err(())
                }]
            });
        }

        let block = try!(block.ok_or(()));

        // Infer list length from index.
        if end_expr.is_none() {
            end_expr = infer_len::infer(&block, &name);
        }

        let end_expr = try!(end_expr.ok_or(()));
        Ok((range, ForN {
            name: name,
            start: start_expr,
            end: end_expr,
            block: block,
            label: label,
            source_range: source_range,
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        if let Some(ref start) = self.start {
            start.resolve_locals(stack, module);
            stack.truncate(st);
        }
        self.end.resolve_locals(stack, module);
        stack.truncate(st);
        stack.push(Some(self.name.clone()));
        self.block.resolve_locals(stack, module);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub block: Block,
    pub label: Option<Arc<String>>,
    pub source_range: Range,
}

impl Loop {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Loop), ()> {
        let start = convert.clone();
        let node = "loop";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut block: Option<Block> = None;
        let mut label: Option<Arc<String>> = None;
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = Block::from_meta_data(
                "block", convert, ignored) {
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

        let block = try!(block.ok_or(()));
        Ok((convert.subtract(start), Loop {
            block: block,
            label: label,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn to_expression(self) -> Expression {
        let source_range = self.source_range;
        Expression::For(Box::new(For {
            block: self.block,
            label: self.label,
            init: Expression::Block(Block {
                expressions: vec![],
                source_range: source_range,
            }),
            step: Expression::Block(Block {
                expressions: vec![],
                source_range: source_range,
            }),
            cond: Expression::Bool(Bool {
                val: true,
                source_range: source_range,
            }),
            source_range: source_range,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Break {
    pub label: Option<Arc<String>>,
    pub source_range: Range,
}

impl Break {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Break), ()> {
        let start = convert.clone();
        let node = "break";
        let start_range = try!(convert.start_node(node));
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
            label: label,
            source_range: convert.source(start).unwrap(),
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Continue {
    pub label: Option<Arc<String>>,
    pub source_range: Range,
}

impl Continue {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Continue), ()> {
        let start = convert.clone();
        let node = "continue";
        let start_range = try!(convert.start_node(node));
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
            label: label,
            source_range: convert.source(start).unwrap(),
        }))
    }
}

#[derive(Debug, Clone)]
pub struct If {
    pub cond: Expression,
    pub true_block: Block,
    pub else_if_conds: Vec<Expression>,
    pub else_if_blocks: Vec<Block>,
    pub else_block: Option<Block>,
    pub source_range: Range,
}

impl If {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, If), ()> {
        let start = convert.clone();
        let node = "if";
        let start_range = try!(convert.start_node(node));
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
                "cond", convert, ignored) {
                convert.update(range);
                cond = Some(val);
            } else if let Ok((range, val)) = Block::from_meta_data(
                "true_block", convert, ignored) {
                convert.update(range);
                true_block = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "else_if_cond", convert, ignored) {
                convert.update(range);
                else_if_conds.push(val);
            } else if let Ok((range, val)) = Block::from_meta_data(
                "else_if_block", convert, ignored) {
                convert.update(range);
                else_if_blocks.push(val);
            } else if let Ok((range, val)) = Block::from_meta_data(
                "else_block", convert, ignored) {
                convert.update(range);
                else_block = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let cond = try!(cond.ok_or(()));
        let true_block = try!(true_block.ok_or(()));
        Ok((convert.subtract(start), If {
            cond: cond,
            true_block: true_block,
            else_if_conds: else_if_conds,
            else_if_blocks: else_if_blocks,
            else_block: else_block,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        self.cond.resolve_locals(stack, module);
        stack.truncate(st);
        self.true_block.resolve_locals(stack, module);
        stack.truncate(st);
        // Does not matter that conditions are resolved before blocks,
        // since the stack gets truncated anyway.
        for else_if_cond in &self.else_if_conds {
            else_if_cond.resolve_locals(stack, module);
            stack.truncate(st);
        }
        for else_if_block in &self.else_if_blocks {
            else_if_block.resolve_locals(stack, module);
            stack.truncate(st);
        }
        if let Some(ref else_block) = self.else_block {
            else_block.resolve_locals(stack, module);
            stack.truncate(st);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Compare {
    pub op: CompareOp,
    pub left: Expression,
    pub right: Expression,
    pub source_range: Range,
}

impl Compare {
    pub fn from_meta_data(
        mut convert: Convert,
        ignored: &mut Vec<Range>)
    -> Result<(Range, Compare), ()> {
        let start = convert.clone();
        let node = "compare";
        let start_range = try!(convert.start_node(node));
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
                "left", convert, ignored) {
                convert.update(range);
                left = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "right", convert, ignored) {
                convert.update(range);
                right = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let op = try!(op.ok_or(()));
        let left = try!(left.ok_or(()));
        let right = try!(right.ok_or(()));
        Ok((convert.subtract(start), Compare {
            op: op,
            left: left,
            right: right,
            source_range: convert.source(start).unwrap(),
        }))
    }

    pub fn resolve_locals(&self, stack: &mut Vec<Option<Arc<String>>>, module: &Module) {
        let st = stack.len();
        self.left.resolve_locals(stack, module);
        stack.truncate(st);
        stack.push(None);
        self.right.resolve_locals(stack, module);
        stack.truncate(st);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompareOp {
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
}

impl CompareOp {
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
