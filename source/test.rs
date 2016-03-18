fn div(x, y) -> {
    if y == 0 {
        return err("division by zero")
    } else {
        return ok(x / y)
    }
}

fn main() {
    println(div(3, 0))
    println(div(2, 3))
}
