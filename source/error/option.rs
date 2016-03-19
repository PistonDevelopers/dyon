fn foo() -> {
    x := none()
    return ok(x?)
}

fn bar() -> {
    return none()
}

fn baz() -> {
    x := bar()?
    return ok(x)
}

fn main() {
    x := unwrap(baz())
    println(x)
}
