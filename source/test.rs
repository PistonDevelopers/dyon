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
    print(", " + f.type)
    println("")
}

fn foo(a: 'b, b) -> {}

fn main() {
    fs := functions()
    n := len(fs)
    type := "loaded"
    // type := "intrinsic"
    for i := 0; i < n; i += 1 {
        if fs[i].type != type { continue }
        print(function: fs[i])
    }
    // println(fs)
}
