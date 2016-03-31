fn main() {
    module := unwrap(load("source/module.rs"))
    call(module, "hello_world", [])
    call(module, "say", ["hi!"])
    call(module, "say_msg_to", ["hi", "john"])

    module2 := unwrap(load(source: "source/module2.rs", imports: [module]))
    call(module2, "main", [])
}
