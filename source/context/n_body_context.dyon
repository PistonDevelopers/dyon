fn start(source) {
    n_body := unwrap(load("source/bench/n_body.rs"))
    m := unwrap(load(source: source, imports: [n_body]))
    println("RUNNING: " + source)
    call(m, "main", [])
    println("THE END")
}
