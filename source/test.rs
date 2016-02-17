/*
fn main() {
    m := load("source/context/n_body_context.rs")
    call(m, "start", ["source/context/n_body_test.rs"])
}
*/

fn main() {
    x := [1, 2]
    y := 3
    loop {
        push(x, y)
        break
    }
    println(x)
}
