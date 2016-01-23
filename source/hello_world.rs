fn id(x) -> { return x }
fn empty() {}
fn fail_1() -> {}
fn fail_2() { return 1 }
fn say_hello(msg, y) -> {
    println(msg)
    return 10 + y
}
fn change(x) { x = 3 }
fn pack(x) -> { return {x: x} }
fn return_local() -> {
    x := 4
    return {x: x}
}
fn return_obj() -> {
    return {x: {x: 5}}
}
fn return_local_clone() -> {
    x := 4
    return {x: clone(x)}
}

fn main() {
    x := 7
    // y := empty()
    // y := fail_1()
    // y := fail_2()
    // y := id(id(x + 1) * 2)
    y := say_hello(x, 5)
    println(y)

    // change(x)
    // println(x)

    // println(pack(x))
    // println(return_local())
    // println(return_obj())
    println(return_local_clone())
}
