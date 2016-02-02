fn receive(channel: 'return) -> {
    return [true, channel]
}

fn check_msg(obj) {
    if obj[0] {
        println(obj[1].msg)
    } else {
        println("<connection closed>")
    }
}

fn close(obj) {
    obj[0] = false
    obj[1] := []
}

fn main() {
    a := {msg: "none"}

    b := receive(a)
    c := {send: a}

    c.send.msg = "hi!"

    check_msg(b) // prints "hi!"

    close(b)

    c.send.msg = "are you there?"

    check_msg(b) // prints "<connection closed>"
}
