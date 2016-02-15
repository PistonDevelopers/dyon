fn bar(b) -> {
    return clone(b)
}

fn foo(a, b) {
    a[0] = bar(b)
}

fn main() {
    a := [0]
    b := 3
    foo(a, b)
}
