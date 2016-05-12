
// fn f() -> vec4 { return (1, 2) }

f() = (1, 2)

fn foo() -> f64 {
    // `f` returns `any`, so the sum of expression gets `any`
    return f() + 1
}

fn main() {
    println(foo())
}
