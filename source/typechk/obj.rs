fn foo() -> {
    x := {a: 0}
    y := {b: 1}
    return x.a + y.b
}

fn main() {
    println(foo())
}
