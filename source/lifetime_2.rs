fn foo(a, b: 'a) {
    a.list = b
}

fn bar(x) {
    y := [3, 4]
    foo(x, y)
}

fn main() {
    x := {list: [1, 2]}
    bar(x)
    z := [5, 6]
    println(x.list)
}
