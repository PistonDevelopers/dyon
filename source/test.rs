fn main() {
    x := err("hi")
    // x = ok({first_name: "Sven", last_name: "Nilsen"})
    println(x)
    x := unwrap(x)
    println(x.first_name)
}
