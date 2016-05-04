
fn foo() -> {
    for i := 0; i < 10; i += 1 {
        return 2
    }
    return 0
}

fn main() {
    println(foo())
}
