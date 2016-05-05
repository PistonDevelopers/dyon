
fn main() {
    m := unwrap(load("source/bench/n_body.rs"))
    call(m, "main", [])
}
