fn start(source) {
    n_body := load("source/bench/n_body.rs")
    m := load(source: source, imports: [n_body])
    println("RUNNING: " + source)
    call(m, "main", [])
    println("THE END")
}
