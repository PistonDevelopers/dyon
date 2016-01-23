extern crate piston_meta;
extern crate range;

use std::sync::Arc;
use self::range::Range;
use self::piston_meta::bootstrap::Convert;
use self::piston_meta::MetaData;

pub fn convert(data: &[Range<MetaData>], ignored: &mut Vec<Range>)
-> Result<Vec<Function>, ()> {
    let mut functions = vec![];
    let mut convert = Convert::new(data);
    loop {
        if let Ok((range, function)) = Function::from_meta_data(convert, ignored) {
            convert.update(range);
            functions.push(function);
        } else if convert.remaining_data_len() > 0 {
            return Err(());
        } else {
            break;
        }
    }
    Ok(functions)
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Arc<String>,
    pub args: Vec<Arg>,
    pub block: Block,
    pub returns: bool,
}

impl Function {
    pub fn from_meta_data(mut convert: Convert, ignored: &mut Vec<Range>)
    -> Result<(Range, Function), ()> {
        let start = convert.clone();
        let node = "fn";
        let start_range = try!(convert.start_node(node));
        convert.update(start_range);

        let mut name: Option<Arc<String>> = None;
        let mut args: Vec<Arg> = vec![];
        let mut block: Option<Block> = None;
        let mut returns = false;
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
                returns = val;
            } else if let Ok((range, val)) = Block::from_meta_data(
                "block", convert, ignored) {
                convert.update(range);
                block = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let name = try!(name.ok_or(()));
        let block = try!(block.ok_or(()));
        Ok((convert.subtract(start), Function {
            name: name,
            args: args,
            block: block,
            returns: returns
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: Arc<String>,
    pub lifetime: Option<Arc<String>>,
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
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("lifetime") {
                convert.update(range);
                lifetime = Some(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let name = try!(name.ok_or(()));
        Ok((convert.subtract(start), Arg {
            name: name,
            lifetime: lifetime
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub expressions: Vec<Expression>,
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
            expressions: expressions
        }))
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Object(Box<Object>),
    Array(Box<Array>),
    Return(Box<Expression>),
    Break(Break),
    Continue(Continue),
    Block(Block),
    Call(Call),
    Item(Item),
    BinOp(Box<BinOpExpression>),
    Assign(Box<Assign>),
    Text(Text),
    Number(Number),
    Bool(Bool),
    For(Box<For>),
    If(Box<If>),
    Compare(Box<Compare>),
}

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
            } else if let Ok((range, val)) = Object::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Object(Box::new(val)));
            } else if let Ok((range, val)) = Array::from_meta_data(
                    convert, ignored) {
                convert.update(range);
                result = Some(Expression::Array(Box::new(val)));
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "return", convert, ignored) {
                convert.update(range);
                result = Some(Expression::Return(Box::new(val)));
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
                result = Some(Expression::Text(Text { text: val }));
            } else if let Ok((range, val)) = convert.meta_f64("num") {
                convert.update(range);
                result = Some(Expression::Number(Number { num: val }));
            } else if let Ok((range, val)) = convert.meta_bool("bool") {
                convert.update(range);
                result = Some(Expression::Bool(Bool { val: val }));
            } else if let Ok((range, val)) = Call::from_meta_data(
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
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let result = try!(result.ok_or(()));
        Ok((convert.subtract(start), result))
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    pub key_values: Vec<(Arc<String>, Expression)>,
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

        Ok((convert.subtract(start), Object { key_values: key_values }))
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
}

#[derive(Debug, Clone)]
pub struct Array {
    pub items: Vec<Expression>,
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

        Ok((convert.subtract(start), Array { items: items }))
    }
}

#[derive(Debug, Clone)]
pub struct Add {
    pub items: Vec<Mul>,
    pub ops: Vec<BinOp>,
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

        Ok((convert.subtract(start), Add { items: items, ops: ops }))
    }

    pub fn to_expression(mut self) -> Expression {
        if self.items.len() == 1 {
            self.items[0].clone().to_expression()
        } else {
            let op = self.ops.pop().unwrap();
            let last = self.items.pop().unwrap();
            Expression::BinOp(Box::new(BinOpExpression {
                op: op,
                left: self.to_expression(),
                right: last.to_expression()
            }))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mul {
    pub items: Vec<MulVar>,
    pub ops: Vec<BinOp>,
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

        Ok((convert.subtract(start), Mul { items: items, ops: ops }))
    }

    pub fn to_expression(mut self) -> Expression {
        if self.items.len() == 1 {
            self.items[0].clone().to_expression()
        } else {
            let op = self.ops.pop().expect("Expected a binary operation");
            let last = self.items.pop().expect("Expected argument");
            Expression::BinOp(Box::new(BinOpExpression {
                op: op,
                left: self.to_expression(),
                right: last.to_expression()
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
                right: a.exp
            })),
            MulVar::Val(a) => a
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pow {
    pub base: Expression,
    pub exp: Expression
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
        Ok((convert.subtract(start), Pow { base: base, exp: exp }))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow
}

#[derive(Debug, Clone)]
pub enum Id {
    String(Arc<String>),
    F64(f64),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: Arc<String>,
    pub ids: Vec<Id>
}

impl Item {
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
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else if let Ok((range, val)) = convert.meta_string("id") {
                convert.update(range);
                ids.push(Id::String(val));
            } else if let Ok((range, val)) = convert.meta_f64("id") {
                convert.update(range);
                ids.push(Id::F64(val));
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "id", convert, ignored) {
                convert.update(range);
                ids.push(Id::Expression(val));
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let name = try!(name.ok_or(()));
        Ok((convert.subtract(start), Item { name: name, ids: ids }))
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub name: Arc<String>,
    pub args: Vec<Expression>,
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
        loop {
            if let Ok(range) = convert.end_node(node) {
                convert.update(range);
                break;
            } else if let Ok((range, val)) = convert.meta_string("name") {
                convert.update(range);
                name = Some(val);
            } else if let Ok((range, val)) = Expression::from_meta_data(
                "arg", convert, ignored) {
                convert.update(range);
                args.push(val);
            } else {
                let range = convert.ignore();
                convert.update(range);
                ignored.push(range);
            }
        }

        let name = try!(name.ok_or(()));
        Ok((convert.subtract(start), Call {
            name: name,
            args: args
        }))
    }
}

#[derive(Debug, Clone)]
pub struct BinOpExpression {
    pub op: BinOp,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub op: AssignOp,
    pub left: Expression,
    pub right: Expression
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
            right: right
        }))
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
}

#[derive(Debug, Clone)]
pub struct Text {
    pub text: Arc<String>,
}

#[derive(Debug, Clone)]
pub struct Bool {
    pub val: bool,
}

#[derive(Debug, Clone)]
pub struct For {
    pub init: Expression,
    pub cond: Expression,
    pub step: Expression,
    pub block: Block,
    pub label: Option<Arc<String>>,
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
            label: label
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub block: Block,
    pub label: Option<Arc<String>>,
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
            label: label
        }))
    }

    pub fn to_expression(self) -> Expression {
        Expression::For(Box::new(For {
            block: self.block,
            label: self.label,
            init: Expression::Block(Block { expressions: vec![] }),
            step: Expression::Block(Block { expressions: vec![] }),
            cond: Expression::Bool(Bool { val: true })
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Break {
    pub label: Option<Arc<String>>,
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
            label: label
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Continue {
    pub label: Option<Arc<String>>,
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
            label: label
        }))
    }
}

#[derive(Debug, Clone)]
pub struct If {
    pub cond: Expression,
    pub true_block: Block,
    pub else_block: Option<Block>,
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
            else_block: else_block
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Compare {
    pub op: CompareOp,
    pub left: Expression,
    pub right: Expression,
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
            right: right
        }))
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
