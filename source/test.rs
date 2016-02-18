/*
fn main() {
    m := load("source/context/n_body_context.rs")
    call(m, "start", ["source/context/n_body_test.rs"])
}
*/

fn main() {
    m := load(source: "source/context/n_body_context.rs", imports: [])
    call(m, "start", ["source/context/n_body_test.rs"])
}
