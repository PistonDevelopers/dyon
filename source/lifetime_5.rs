fn foo(a, b: 'a) {
    a.list = b
}

fn bar(x) {
    y := [3, 4]
    foo(x, y)
}

fn main() {
    z := {list: [1, 2]}
    bar(z)
    println(z.list)
}
