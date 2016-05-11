fn foo() -> opt[bool] {
    return some(true)
}

fn main() {
    if foo() {
        println("oh?")
    }
}
