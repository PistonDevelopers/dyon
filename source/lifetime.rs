fn foo(a: 'return) -> { return a }
// fn foo(a) -> { return a } // ERROR: Argument `a` does not have lifetime `return`

fn main() {
    x := 2
    println(foo(x))
}
