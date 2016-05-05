fn up_to(n) -> {
    x := []
    for i := 0; i < n; i += 1 {
        push(mut x, clone(i))
    }
    return clone(x)
}

fn say_msg_to(msg, person) {
    print(person + "! ")
    println(msg)
}

fn main() {
    n := 3
    println(up(to: n))

    // Normal call syntax.
    say_msg_to("hi!", "you there")
    // Named arguments call syntax.
    say(msg: "hi!", to: "you there")
}
