fn foo() -> bool {
    return load("source/test.rs")
}

fn main() {
    if foo() {
        println("oh?")
    }
}
