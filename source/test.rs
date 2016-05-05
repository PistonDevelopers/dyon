
fn foo(mut list) {
    push(mut list, 3)
}

fn bar(mut list: 'return) -> {
    return pop(mut list)
}

fn main() {
    list := []
    foo(mut list)
    println(list)
    bar(mut list)
    println(list)
}
