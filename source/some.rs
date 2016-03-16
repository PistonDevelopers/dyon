fn foo() -> {
    x := [5]
    return clone(some(x))
}

fn main() {
    x := foo()
    println(x)
}
