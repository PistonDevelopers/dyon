fn say_to(msg, person) {
    print(person + "! ")
    println(msg)
}

fn main() {
    // Rusty call syntax.
    say_to("hi!", "you there")
    // SmallTalk call syntax.
    (say: "hi!" to: "you there")
}
