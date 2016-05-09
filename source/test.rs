fn main() {
    println(any i 3 { i == 7 })
    // println(any(3))
    println(all i 3 { i != 5 })
    // println(all(3))
}

fn any(n) -> {
    return = false
    for i n {
        if i == 2 { return true }
    }
}

fn all(n) -> {
    return = true
    for i n {
        if i != 5 { return false }
    }
}
