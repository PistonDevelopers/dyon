fn main() {
    x := {x: {x: 0}}
    for i := 0; i < 1; i += 1
    {
        y := 5
        x.x = {x: y}
    }
    debug()
    println(x.x)
}
