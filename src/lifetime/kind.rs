
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Ns,
    Uses,
    Use,
    Fn,
    Arg,
    Current,
    Block,
    Expr,
    Add,
    Mul,
    Pow,
    Base,
    Exp,
    Val,
    Call,
    CallArg,
    Assign,
    Left,
    Right,
    Item,
    ItemExtra,
    Return,
    Object,
    Array,
    ArrayItem,
    ArrayFill,
    Fill,
    N,
    KeyValue,
    For,
    ForN,
    ForIn,
    Sum,
    SumIn,
    SumVec4,
    Prod,
    ProdIn,
    ProdVec4,
    Min,
    MinIn,
    Max,
    MaxIn,
    Sift,
    Any,
    AnyIn,
    All,
    Vec4UnLoop,
    Start,
    End,
    Init,
    Cond,
    Iter,
    ElseIfCond,
    ElseIfBlock,
    Step,
    Compare,
    If,
    TrueBlock,
    ElseBlock,
    Loop,
    Id,
    Break,
    Continue,
    Norm,
    UnOp,
    Vec4,
    X,
    Y,
    Z,
    W,
    Type,
    Arr,
    Opt,
    Res,
    RetType,
    ReturnVoid,
    Go,
    Swizzle,
    Sw0,
    Sw1,
    Sw2,
    Sw3,
    Link,
    LinkFor,
    LinkItem,
    Closure,
    CallClosure,
    ClosureType,
    ClArg,
    ClRet,
    Grab,
    TryExpr,
    In,
}

impl Kind {
    pub fn new(name: &str) -> Option<Kind> {
        Some(match name {
            "ns" => Kind::Ns,
            "uses" => Kind::Uses,
            "use" => Kind::Use,
            "fn" => Kind::Fn,
            "arg" => Kind::Arg,
            "current" => Kind::Current,
            "block" => Kind::Block,
            "expr" => Kind::Expr,
            "add" => Kind::Add,
            "mul" => Kind::Mul,
            "pow" => Kind::Pow,
            "base" => Kind::Base,
            "exp" => Kind::Exp,
            "val" => Kind::Val,
            "call" => Kind::Call,
            "call_arg" => Kind::CallArg,
            "named_call" => Kind::Call,
            "assign" => Kind::Assign,
            "left" => Kind::Left,
            "right" => Kind::Right,
            "item" => Kind::Item,
            "item_extra" => Kind::ItemExtra,
            "return" => Kind::Return,
            "object" => Kind::Object,
            "array" => Kind::Array,
            "array_item" => Kind::ArrayItem,
            "array_fill" => Kind::ArrayFill,
            "fill" => Kind::Fill,
            "n" => Kind::N,
            "key_value" => Kind::KeyValue,
            "for" => Kind::For,
            "for_n" => Kind::ForN,
            "for_in" => Kind::ForIn,
            "sum" => Kind::Sum,
            "sum_in" => Kind::SumIn,
            "sum_vec4" => Kind::SumVec4,
            "prod" => Kind::Prod,
            "prod_in" => Kind::ProdIn,
            "prod_vec4" => Kind::ProdVec4,
            "min" => Kind::Min,
            "min_in" => Kind::MinIn,
            "max" => Kind::Max,
            "max_in" => Kind::MaxIn,
            "sift" => Kind::Sift,
            "start" => Kind::Start,
            "any" => Kind::Any,
            "any_in" => Kind::AnyIn,
            "all" => Kind::All,
            "vec4_un_loop" => Kind::Vec4UnLoop,
            "end" => Kind::End,
            "init" => Kind::Init,
            "cond" => Kind::Cond,
            "iter" => Kind::Iter,
            "else_if_cond" => Kind::ElseIfCond,
            "else_if_block" => Kind::ElseIfBlock,
            "step" => Kind::Step,
            "compare" => Kind::Compare,
            "if" => Kind::If,
            "true_block" => Kind::TrueBlock,
            "else_block" => Kind::ElseBlock,
            "loop" => Kind::Loop,
            "id" => Kind::Id,
            "break" => Kind::Break,
            "continue" => Kind::Continue,
            "norm" => Kind::Norm,
            "unop" => Kind::UnOp,
            "vec4" => Kind::Vec4,
            "x" => Kind::X,
            "y" => Kind::Y,
            "z" => Kind::Z,
            "w" => Kind::W,
            "type" => Kind::Type,
            "arr" => Kind::Arr,
            "opt" => Kind::Opt,
            "res" => Kind::Res,
            "ret_type" => Kind::RetType,
            "return_void" => Kind::ReturnVoid,
            "go" => Kind::Go,
            "swizzle" => Kind::Swizzle,
            "sw0" => Kind::Sw0,
            "sw1" => Kind::Sw1,
            "sw2" => Kind::Sw2,
            "sw3" => Kind::Sw3,
            "link" => Kind::Link,
            "link_for" => Kind::LinkFor,
            "link_item" => Kind::LinkItem,
            "closure" => Kind::Closure,
            "call_closure" => Kind::CallClosure,
            "named_call_closure" => Kind::CallClosure,
            "closure_type" => Kind::ClosureType,
            "cl_arg" => Kind::ClArg,
            "cl_ret" => Kind::ClRet,
            "grab" => Kind::Grab,
            "try_expr" => Kind::TryExpr,
            "in" => Kind::In,
            _ => return None
        })
    }

    /// A loop can infer range from the body using variable.
    pub fn is_decl_loop(&self) -> bool {
        use self::Kind::*;

        match *self {
            ForN | Sum | Prod | SumVec4 | Min | Max | Sift
            | Any | All | LinkFor => true,
            _ => false
        }
    }

    /// An in-loop receives an object from a receiver channel.
    pub fn is_in_loop(&self) -> bool {
        use self::Kind::*;

        match *self {
            ForIn | SumIn | ProdIn | MinIn | MaxIn | AnyIn => true,
            _ => false
        }
    }

    /// An un-loop has fixed range and replaces variable in body.
    pub fn is_decl_un_loop(&self) -> bool {
        match *self {
            Kind::Vec4UnLoop => true,
            _ => false
        }
    }

    pub fn is_block(&self) -> bool {
        use self::Kind::*;

        match *self {
            Block | ElseIfBlock |
            TrueBlock | ElseBlock => true,
            _ => false
        }
    }
}
