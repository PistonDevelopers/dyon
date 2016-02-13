fn main() {
    module := load("source/module.rs")
    call(module, "hello_world", [])
    call(module, "say", ["hi!"])

    // module2 := load(source: "source/module2.rs", imports: [module])
}
