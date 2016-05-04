
fn foo_one_two(mut a, b) {
    a[0] = clone(b)
}

fn foo_one_two(a, b) {
    a[0] = clone(b)
}

fn main() {
    a := [4]
    b := 5
    foo(one: mut a, two: b)
    println(a)
}
