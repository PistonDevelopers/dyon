fn main() {
    x := 0
    y := 1
    z := if false { x } else { y }
    // z := { y }
    // z := y
    // debug()
    println(z)
}
