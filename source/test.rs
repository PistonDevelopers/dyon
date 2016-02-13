fn main() {
    module := load("source/module.rs")
    call(module, "hello_world", [])
    call(module, "say", ["hi!"])
}
