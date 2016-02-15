fn one() -> {return 1}
fn bar(x: 'return) -> {return [x]}
fn foo() -> {
    return bar(0)
}
fn main() {
    println(foo())
}
