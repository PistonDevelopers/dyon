use std::collections::HashMap;

pub fn standard() -> HashMap<&'static str, Intrinsic> {
    let mut i: HashMap<&'static str, Intrinsic> = HashMap::new();
    i.insert("println", PRINTLN);
    i.insert("print", PRINT);
    i.insert("clone", CLONE);
    i.insert("debug", DEBUG);
    i.insert("backtrace", BACKTRACE);
    i.insert("sleep", SLEEP);
    i.insert("round", ROUND);
    i.insert("random", RANDOM);
    i.insert("read_number", READ_NUMBER);
    i.insert("read_line", READ_LINE);
    i.insert("len", LEN);
    i.insert("push", PUSH);
    i.insert("trim_right", TRIM_RIGHT);
    i.insert("to_string", TO_STRING);
    i.insert("typeof", TYPEOF);
    i.insert("sqrt", SQRT);
    i.insert("sin", SIN);
    i.insert("asin", ASIN);
    i.insert("cos", COS);
    i.insert("acos", ACOS);
    i.insert("tan", TAN);
    i.insert("atan", ATAN);
    i.insert("exp", EXP);
    i.insert("ln", LN);
    i.insert("log2", LOG2);
    i.insert("log10", LOG10);
    i.insert("random", RANDOM);
    i.insert("load", LOAD);
    i.insert("load_source_imports", LOAD_SOURCE_IMPORTS);
    i.insert("call", CALL);
    i
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArgConstraint {
    Arg(usize),
    Return,
    Default,
}

#[derive(Debug, Copy, Clone)]
pub struct Intrinsic {
    pub arg_constraints: &'static [ArgConstraint],
    pub returns: bool,
}

static PRINTLN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static PRINT: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static CLONE: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static DEBUG: Intrinsic = Intrinsic {
    arg_constraints: &[],
    returns: false
};

static BACKTRACE: Intrinsic = Intrinsic {
    arg_constraints: &[],
    returns: false
};

static SLEEP: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static ROUND: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static RANDOM: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static READ_NUMBER: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static READ_LINE: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TRIM_RIGHT: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LEN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static PUSH: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default, ArgConstraint::Arg(0)],
    returns: false
};

static SQRT: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static ASIN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static SIN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static COS: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static ACOS: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TAN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static ATAN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static EXP: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOG2: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOG10: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TO_STRING: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TYPEOF: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOAD: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOAD_SOURCE_IMPORTS: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default; 2],
    returns: true
};

static CALL: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default; 3],
    returns: true
};
