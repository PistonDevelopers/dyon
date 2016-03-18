fn foo() -> {
    return unwrap(ok(none()))
}

fn main() {
    x := foo()
    println(x)
}
