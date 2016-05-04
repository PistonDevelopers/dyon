
fn foo(mut a, b) {
    a[0] = clone(b)
}

fn bar(a, b) {
    foo(mut a, b)
}

fn main() {
    a := [4]
    b := 5
    bar(a, b)
}
