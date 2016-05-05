fn foo(mut a, b: 'a) {
    a = b
}

fn main() {
    b := [[5]]
    a := [[]]
    foo(mut a, b)
    println(a)
    // debug()

    push(mut a, b)
    // b = [3]
    println(a)
}

/*
fn main() {
    b := {x: {}}
    a := {x: 6}
    foo(mut a, b)
    println(a)

    a.x = b
    println(a)
}
*/

/*
fn main() {
    b := some({x: {}})
    a := some({x: 6})
    foo(mut a, b)
    println(a)
    debug()

    b = none()
    println(a)
}
*/

/*
fn main() {
    b := ok({x: {}})
    a := ok({x: 6})
    foo(mut a, b)
    println(a)
    debug()

    b = err("no")
    println(a)
}
*/

/*
fn main() {
    a := [5]
    b := [6]
    b = a
    b[0] = 3
    println(a)
}
*/
