fn foo_val(a) -> {
    // return err("hi")
    return ok(a + 3)
}

fn bar_val(a) -> {
    x := if a == 0 { ok(1) } else { foo(val: a) }?
    return ok(x + 2)
}

fn main() {
    x := bar(val: 0)
    println(x)
}
