fn foo() -> {
    return err("something wrong happened")
}

fn bar() -> {
    x := foo()?
    return ok(x + 1)
}

fn baz() -> {
    x := bar()?
    return ok(x + 1)
}

fn main() {
    println(unwrap(baz()))
}
