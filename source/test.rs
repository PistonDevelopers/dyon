/*
fn main() {
    m := load("source/context/n_body_context.rs")
    call(m, "start", ["source/context/n_body_test.rs"])
}
*/

fn foo() -> {
    x := 4
    return [5; x]
}

fn main() {
    println(foo())
}
