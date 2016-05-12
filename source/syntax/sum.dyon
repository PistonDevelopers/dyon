fn main() {
    list := [3, 4, 5, 12, 42]
    n := len(list)

    println("sum: " + to_string(∑ i n { list[i] }))
    println("avg: " + to_string(∑ i n { list[i] } / n))
    println("odd: " + to_string(∑ i n { list[i] % 2 }))
    println("sum >10: " + to_string(∑ i n {
        if list[i] > 10 { list[i] } else { continue }
    }))
    println("stop at 5: " + to_string(∑ i n {
        if list[i] == 5 { break }
        list[i]
    }))
    println("sim prod: " + to_string(exp(∑ i n { ln(list[i]) })))
    println("sim any >30: " + to_string(0 != ∑ i n {
        if list[i] > 30 { 1 } else { continue }
    }))
    println("sim all <50: " + to_string(n == ∑ i n {
        if list[i] < 50 { 1 } else { continue }
    }))
}
