use std::cmp::{PartialOrd, Ordering};
use super::node::Node;
use super::ArgNames;

/// Describes the lifetime of a variable.
/// When a lifetime `a` > `b` it means `a` outlives `b`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lifetime {
    /// Return value with optional list of arguments that outlives other arguments.
    Return(Vec<usize>),
    /// An argument outlives other arguments, but does not outlive the return.
    Argument(Vec<usize>),
    /// Local variable.
    Local(usize),
    /// Current variable.
    ///
    /// Is equal to itself and outlives local variables.
    ///
    /// Unknown to return because lifetime checker infers
    /// lifetime or return value from argument, which does not work
    /// with current objects.
    Current(usize),
}

impl PartialOrd for Lifetime {
    fn partial_cmp(&self, other: &Lifetime) -> Option<Ordering> {
        use self::Lifetime::*;

        Some(match (self, other) {
            (&Current(_), &Local(_)) => Ordering::Greater,
            (&Local(_), &Current(_)) => Ordering::Less,
            (&Current(a), &Current(b)) if a == b => Ordering::Equal,
            (&Current(_), _) => return None,
            (_, &Current(_)) => return None,
            (&Local(a), &Local(b)) => b.cmp(&a),
            (&Return(_), &Local(_)) => Ordering::Greater,
            (&Local(_), &Return(_)) => Ordering::Less,
            (&Return(ref a), &Return(ref b)) => {
                match (a.len(), b.len()) {
                    (0, 0) => Ordering::Equal,
                    (0, _) => Ordering::Less,
                    (_, 0) => Ordering::Greater,
                    (_, _) => {
                        return compare_argument_outlives(a, b);
                    }
                }
            }
            (&Argument(_), &Local(_)) => Ordering::Greater,
            (&Local(_), &Argument(_)) => Ordering::Less,
            (&Return(_), &Argument(_)) => return None,
            (&Argument(_), &Return(_)) => return None,
            (&Argument(ref a), &Argument(ref b)) => {
                return compare_argument_outlives(a, b);
            }
        })
    }
}

/// Takes two lists of arguments.
/// If they have any argument in common, the longer list outlives the shorter.
/// If they have no argument in common, it is not known whether one outlives
/// the other.
fn compare_argument_outlives(a: &[usize], b: &[usize]) -> Option<Ordering> {
    for &i in a {
        for &j in b {
            if i == j {
                return Some(a.len().cmp(&b.len()));
            }
        }
    }
    None
}

/// Gets the lifetime of a function argument.
pub fn arg_lifetime(
    declaration: usize,
    arg: &Node,
    nodes: &[Node],
    arg_names: &ArgNames
) -> Option<Lifetime> {
    Some(if let Some(ref lt) = arg.lifetime {
        if &**lt == "return" {
            return Some(Lifetime::Return(vec![declaration]));
        } else {
            // Resolve lifetimes among arguments.
            let parent = arg.parent.expect("Expected parent");
            let mut args: Vec<usize> = vec![];
            args.push(declaration);
            let mut name = lt.clone();
            loop {
                let (arg, _) = *arg_names.get(&(parent, name))
                    .expect("Expected argument name");
                args.push(arg);
                if let Some(ref lt) = nodes[arg].lifetime {
                    if &**lt == "return" {
                        // Lifetimes outlive return.
                        return Some(Lifetime::Return(args));
                    }
                    name = lt.clone();
                } else {
                    break;
                }
            }
            Lifetime::Argument(args)
        }
    } else {
        Lifetime::Argument(vec![declaration])
    })
}

pub fn compare_lifetimes(
    l: &Option<Lifetime>,
    r: &Option<Lifetime>,
    nodes: &[Node]
) -> Result<(), String> {
    match (l, r) {
        (&Some(ref l), &Some(ref r)) => {
            match l.partial_cmp(&r) {
                Some(Ordering::Greater) | Some(Ordering::Equal) => {
                    match *r {
                        Lifetime::Local(r) => {
                            // This gets triggered in cases like these:
                            /*
                            fn main() {
                                a := [[]]
                                b := [3]    // <--- declared after 'a'
                                a[0] = b    // <--- attempting to put 'b' inside 'a'
                            }
                            */
                            return Err(format!("`{}` does not live long enough",
                                nodes[r].name().expect("Expected name")));
                        }
                        Lifetime::Argument(ref r) => {
                            return Err(format!("`{}` does not live long enough",
                                nodes[r[0]].name().expect("Expected name")));
                        }
                        Lifetime::Current(r) => {
                            return Err(format!("`{}` does not live long enough",
                                nodes[r].name().expect("Expected name")));
                        }
                        _ => unimplemented!()
                    }
                }
                None => {
                    match (l, r) {
                        (&Lifetime::Argument(ref l), &Lifetime::Argument(ref r)) => {
                            // TODO: Report function name for other cases.
                            let func = nodes[nodes[r[0]].parent.unwrap()]
                                .name().unwrap();
                            return Err(format!("Function `{}` requires `{}: '{}`",
                                func,
                                nodes[r[0]].name().expect("Expected name"),
                                nodes[l[0]].name().expect("Expected name")));
                        }
                        (&Lifetime::Argument(ref l), &Lifetime::Return(ref r)) => {
                            if !r.is_empty() {
                                return Err(format!("Requires `{}: '{}`",
                                    nodes[r[0]].name().expect("Expected name"),
                                    nodes[l[0]].name().expect("Expected name")));
                            } else {
                                unimplemented!();
                            }
                        }
                        (&Lifetime::Return(ref l), &Lifetime::Return(ref r)) => {
                            if !l.is_empty() && !r.is_empty() {
                                return Err(format!("Requires `{}: '{}`",
                                    nodes[r[0]].name().expect("Expected name"),
                                    nodes[l[0]].name().expect("Expected name")));
                            } else {
                                unimplemented!();
                            }
                        }
                        (&Lifetime::Return(ref l), &Lifetime::Argument(ref r)) => {
                            if l.is_empty() {
                                let last = *r.last().expect("Expected argument index");
                                return Err(format!("Requires `{}: 'return`",
                                    nodes[last].name().expect("Expected name")));
                            } else {
                                return Err(format!("`{}` does not live long enough",
                                    nodes[r[0]].name().expect("Expected name")));
                            }
                        }
                        (&Lifetime::Current(n), _) => {
                            return Err(format!("`{}` is a current object, use `clone(_)`",
                                nodes[n].name().expect("Expected name")));
                        }
                        (_, &Lifetime::Current(n)) => {
                            return Err(format!("`{}` is a current object, use `clone(_)`",
                                nodes[n].name().expect("Expected name")));
                        }
                        x => panic!("Unknown case {:?}", x)
                    }
                }
                _ => {}
            }
        }
        // TODO: Handle other cases.
        _ => {}
    }
    Ok(())
}
