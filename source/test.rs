fn main() {
    x := {0: 0, 1: 1, 2: 2, 3: 3, 4: 4}
    for i := 0; i < 5; i += 1 {
        i := to_string(i)
        println(x[i])
        x[i] += 1
    }
    println(x)
}
