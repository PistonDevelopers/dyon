fn foo() -> {
    return err("something wrong happened")
    // return ok(5)
}

fn main() {
    x := foo()
    if is_err(x) {
        println(unwrap_err(x))
    } else {
        println(unwrap(x))
    }
}
