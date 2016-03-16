fn main() {
    context := load("source/functions/context.rs")
    print_context := load(
        source: "source/functions/print_context.rs",
        imports: [context]
    )
    call(print_context, "main", [])
}
