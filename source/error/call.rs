fn foo() -> {
    // return err("hi")
    return ok(3)
}

fn bar() -> {
    x := foo()?
    return ok(x + 2)
}

fn main() {
    x := bar()
    println(x)
}
