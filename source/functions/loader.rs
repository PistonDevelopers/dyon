fn main() {
    functions := unwrap(load("source/functions/functions.rs"))
    print := unwrap(load(
        source: "source/functions/print.rs",
        imports: [functions]
    ))
    call(print, "main", [])
}
