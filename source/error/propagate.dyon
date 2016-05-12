fn foo() -> {
    // x := ok(2)
    x := err("hi")
    y := x? + 3
    return ok(y)
}

fn foo2() -> {
    // x := {a: ok(2)}
    x := {a: err("hi")}
    y := x.a?
    return ok(y)
}

fn foo3() -> {
    // x := {a: {b: ok(2)}}
    x := {a: {b: err("error")}}
    y := x.a.b?
    return ok(y)
}

fn foo4() -> {
    // x := {a: ok({b: ok(8)})}
    // x := {a: ok({b: err("error 1")})}
    x := {a: err("error 2")}
    y := x.a?.b?
    return ok(y)
}

fn foo5() -> {
    // x := [{a: ok([ok(3)])}]
    x := [{a: err("error")}]
    y := x[0].a?[0]?
    return ok(y)
}

/*
fn bar() {
    x := err("hi")
    x?
}
*/

fn main() {
    x := foo5()
    println(x)
    // bar()
}
