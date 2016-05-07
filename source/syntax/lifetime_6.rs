fn bar(b) -> {
    return clone(b)
}

fn foo(mut a, b) {
    a[0] = bar(b)
}

fn main() {
    a := [0]
    b := 3
    foo(mut a, b)
}
