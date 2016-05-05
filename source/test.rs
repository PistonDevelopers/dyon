/*
fn foo(mut a, b: 'a) {
    a = b
}

fn main() {
    b := [[5]]
    a := [[]]
    foo(mut a, b)
    println(a)
    debug()

    push(mut a, b)
    println(a)
}
*/

fn main() {
    a := [5]
    b := [6]
    b = a
    debug()
}
