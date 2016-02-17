/*
fn main() {
    m := load("source/context/n_body_context.rs")
    call(m, "start", ["source/context/n_body_test.rs"])
}
*/

fn main() {
    print("type a number: ")
    err := "It must be a number!"
    x := read_number(err)
    println("you typed: " + to_string(x))
}
