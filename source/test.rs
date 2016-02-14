fn main() {
    m := load("source/bench/add.rs")
    for i := 0; i < 100; i += 1 {
        call(m, "main", [])
    }
}
