fn main() {
    x := if false {0} else if true {1} else {2}
    println(x)
    y := if false {0} else if true {1}
    println(y)

    if false {
    } else if false {
    } else if true {
        println("hi")
    } else {
    }
}
