/*
fn main() {
    m := load("source/context/n_body_context.rs")
    call(m, "start", ["source/context/n_body_test.rs"])
}
*/

fn foo() -> {
    x := 5
    return [5, 3 + x]
}

fn main() {
    println(foo())
}
