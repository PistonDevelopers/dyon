fn receive(channel: 'return) -> {
    return {open: true, receive: channel}
}

fn check_msg(obj) {
    if obj.open {
        println(obj.receive.msg)
    } else {
        println("<connection closed>")
    }
}

fn close(obj) {
    obj.open = false
    obj.receive := []
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
