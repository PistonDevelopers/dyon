fn main() {
    pos := {x: 0}
    for i := 0; i < 100_000; i += 1 {
        pos.x += 1
    }
}
