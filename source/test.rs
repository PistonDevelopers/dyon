
fn foo(mut a, b) {
    a[0] = clone(b)
}

fn main() {
    a := [4]
    b := 5
    foo(mut a, b)
    println(a)
}
