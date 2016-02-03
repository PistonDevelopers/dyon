fn main() {
    x := [0, 1, 2, 3, 4]
    for i := 0; i < len(x); i += 1 {
        x[i] += 1
    }
    println(x)
}
