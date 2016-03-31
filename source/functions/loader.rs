fn main() {
    context := unwrap(load("source/functions/context.rs"))
    print_context := unwrap(load(
        source: "source/functions/print_context.rs",
        imports: [context]
    ))
    call(print_context, "main", [])
}
