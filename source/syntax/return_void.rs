fn foo() -> {
    return = "hi!"
    return
}

fn main() {
    x := foo()
    println(x)
    for i := 0; i < 100; i += 1 {
        if i > 14 {
            return
        }
        println(i)
    }
}
