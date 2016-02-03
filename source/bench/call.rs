fn foo() {}

fn main() {
    for i := 0; i < 100_000; i += 1 {
        foo()
    }
}
