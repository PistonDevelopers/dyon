fn main() {
    n := 100
    min := unwrap(min i n { f(i, n) })
    max := unwrap(max i n { f(i, n) })
    println("min: " + to_string(min))
    println("max: " + to_string(max))
}

fn f(i, n) -> {
    x := (i - n / 2) / n
    x -= 0.8
    return x^2
}
