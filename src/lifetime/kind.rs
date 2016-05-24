
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
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
    Sum,
    SumVec4,
    Min,
    Max,
    Sift,
    Any,
    All,
    Vec4UnLoop,
    Start,
    End,
    Init,
    Cond,
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
}

impl Kind {
    pub fn new(name: &str) -> Option<Kind> {
        Some(match name {
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
            "sum" => Kind::Sum,
            "sum_vec4" => Kind::SumVec4,
            "min" => Kind::Min,
            "max" => Kind::Max,
            "sift" => Kind::Sift,
            "start" => Kind::Start,
            "any" => Kind::Any,
            "all" => Kind::All,
            "vec4_un_loop" => Kind::Vec4UnLoop,
            "end" => Kind::End,
            "init" => Kind::Init,
            "cond" => Kind::Cond,
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
            _ => return None
        })
    }

    /// A loop can infer range from the body using variable.
    pub fn is_decl_loop(&self) -> bool {
        use self::Kind::*;

        match *self {
            ForN | Sum | SumVec4 | Min | Max | Sift
            | Any | All => true,
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
