fn bar() -> {
    return err(3)
}

fn foo(x) {
    y := bar()?
    println(y)
}

fn main() {
    // x := err(5)?
    x := {a: ok([err(2)])}
    y := x.a?[0]?
    println(y)
    // foo({a: err(5)})
}
