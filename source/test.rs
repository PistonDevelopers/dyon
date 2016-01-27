fn main() {
    a := {x: 0}
    a.x = 2
    println(a.x)

    b := [[0, 1]]
    b[0][1] = 7
    println(b)

    c := {x: {y: {z: 0}}}
    c.x.y.z = 7
    println(c)
}
