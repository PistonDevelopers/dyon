/*
fn main() {
    m := load("source/context/n_body_context.rs")
    call(m, "start", ["source/context/n_body_test.rs"])
}
*/

/*
fn main() {
    m := load(source: "source/context/n_body_context.rs", imports: [])
    call(m, "start", ["source/context/n_body_test.rs"])
}
*/

/*
fn main() {
    x := {a: "hi", b: "hi"}
    y := {b: "hi", a: "hi"}
    println(x == y)
}
*/

fn main() {
    x := ["hi", "hi"]
    y := ["hi", "hi"]
    println(x == y)
}
