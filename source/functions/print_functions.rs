fn print_function(f) {
    print(f.name)
    print("(")
    n := len(f.arguments)
    for i := 0; i < n; i += 1 {
        print(f.arguments[i].name)
        if f.arguments[i].lifetime != none() {
            print(": '" + unwrap(f.arguments[i].lifetime))
        }
        if (i + 1) < n {
            print(", ")
        }
    }
    print(")")
    if f.returns {
        print(" ->")
    }
    println("")
}

fn foo(a: 'b, b) -> {}

fn main() {
    fs := functions()
    n := len(fs)
    for i := 0; i < n; i += 1 {
        print(function: fs[i])
    }
}
